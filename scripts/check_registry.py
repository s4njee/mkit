#!/usr/bin/env python3
"""Validate the checked-in component registry against workspace metadata."""

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "registry/registry.json"
VERSION_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$")
REQUIRED_DEPENDENCIES = {"mkit-core", "gpui-pre"}


def fail(message: str) -> None:
    raise ValueError(message)


def repository_file(value: object, component_name: str, field: str) -> Path:
    if not isinstance(value, str) or not value:
        fail(f"{component_name}: {field} must be a repository path")
    candidate = (ROOT / value).resolve()
    if ROOT not in candidate.parents or not candidate.is_file():
        fail(f"{component_name}: {field} must name an existing file inside the repository")
    return candidate


def validate(registry: dict, workspace_version: str, required_versions: dict[str, str]) -> None:
    if registry.get("registry_format_version") != 1:
        fail("registry_format_version must be 1")
    if registry.get("release_version") != workspace_version:
        fail("release_version must match workspace.package.version")
    distribution = registry.get("distribution", {})
    if distribution != {"kind": "repository", "path": "registry/", "published": False}:
        fail("distribution must describe the in-repository registry")

    components = registry.get("components")
    if not isinstance(components, list) or not components:
        fail("components must be a non-empty array")
    names = set()
    component_by_name = {}
    for component in components:
        name = component.get("name")
        if not isinstance(name, str) or not re.fullmatch(r"[a-z][a-z0-9-]*", name):
            fail("component name must be lowercase kebab-case")
        if name in names:
            fail(f"duplicate component: {name}")
        names.add(name)
        component_by_name[name] = component
        version = component.get("version")
        if not isinstance(version, str) or not VERSION_RE.fullmatch(version):
            fail(f"{name}: invalid version")
        if version != workspace_version:
            fail(f"{name}: version must match workspace.package.version")
        status = component.get("status")
        if status not in {"draft_spec_only", "implementation_in_progress", "source_ready"}:
            fail(f"{name}: invalid status")
        source_files = component.get("source_files")
        if not isinstance(source_files, list) or any(not isinstance(path, str) for path in source_files):
            fail(f"{name}: source_files must be an array of paths")
        if status == "draft_spec_only" and source_files:
            fail(f"{name}: draft specs cannot claim source files")
        if status in {"implementation_in_progress", "source_ready"} and not source_files:
            fail(f"{name}: {status} requires source files")
        if status in {"implementation_in_progress", "source_ready"}:
            for source_path in source_files:
                repository_file(source_path, name, "source_files entry")
        for field in ("spec", "conformance_manifest"):
            repository_file(component.get(field), name, field)
        component_dependencies = component.get("component_dependencies")
        if not isinstance(component_dependencies, list) or any(not isinstance(dep, str) for dep in component_dependencies):
            fail(f"{name}: component_dependencies must be an array of component names")
        if len(component_dependencies) != len(set(component_dependencies)):
            fail(f"{name}: duplicate component dependency")
        dependencies = component.get("dependencies")
        if not isinstance(dependencies, list):
            fail(f"{name}: dependencies must be an array")
        dependency_versions = {}
        for dependency in dependencies:
            dep_name, dep_version = dependency.get("name"), dependency.get("version")
            if not isinstance(dep_name, str) or not isinstance(dep_version, str) or not VERSION_RE.fullmatch(dep_version):
                fail(f"{name}: invalid dependency")
            if dep_name in dependency_versions:
                fail(f"{name}: duplicate dependency {dep_name}")
            dependency_versions[dep_name] = dep_version
        if not REQUIRED_DEPENDENCIES <= dependency_versions.keys():
            fail(f"{name}: dependencies must include mkit-core and gpui-pre")
        if dependency_versions["mkit-core"] != required_versions["mkit-core"]:
            fail(f"{name}: mkit-core version must match the workspace dependency")
        if dependency_versions["gpui-pre"] != required_versions["gpui-pre"]:
            fail(f"{name}: gpui-pre version must match the workspace dependency")

    graph = {name: component["component_dependencies"] for name, component in component_by_name.items()}
    for name, dependencies in graph.items():
        for dependency in dependencies:
            if dependency not in graph:
                fail(f"{name}: unknown component dependency {dependency}")
            if dependency == name:
                fail(f"{name}: component dependency cycle")

    visiting = set()
    visited = set()

    def visit(name: str) -> None:
        if name in visiting:
            fail(f"component dependency cycle involving {name}")
        if name in visited:
            return
        visiting.add(name)
        for dependency in graph[name]:
            visit(dependency)
        visiting.remove(name)
        visited.add(name)

    for name in graph:
        visit(name)


def main() -> int:
    try:
        registry = json.loads(REGISTRY_PATH.read_text())
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
        workspace_data = workspace["workspace"]
        declared = workspace_data["dependencies"]
        gpui = declared["gpui_pre"]
        validate(
            registry,
            workspace_data["package"]["version"],
            {
                "mkit-core": declared["mkit-core"]["version"],
                "gpui-pre": gpui["version"].lstrip("="),
            },
        )
    except (OSError, json.JSONDecodeError, tomllib.TOMLDecodeError, KeyError, ValueError) as error:
        print(f"registry check failed: {error}", file=sys.stderr)
        return 1
    print(f"registry check passed: {len(registry['components'])} components")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
