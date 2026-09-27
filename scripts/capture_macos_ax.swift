// Capture the native accessibility tree of a running GPUI app on macOS.
// Usage: swift scripts/capture_macos_ax.swift <pid> > ax-tree.json
import ApplicationServices
import Foundation

struct Node: Encodable {
    let role: String
    let subrole: String?
    let title: String?
    let value: String?
    let help: String?
    let enabled: Bool?
    let children: [Node]
}

func attribute(_ element: AXUIElement, _ key: String) -> CFTypeRef? {
    var result: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, key as CFString, &result) == .success else {
        return nil
    }
    return result
}

func stringAttribute(_ element: AXUIElement, _ key: String) -> String? {
    attribute(element, key).map { String(describing: $0) }
}

func capture(_ element: AXUIElement, depth: Int) -> Node {
    var children: [Node] = []
    if depth < 12, let elements = attribute(element, kAXChildrenAttribute) as? [AXUIElement] {
        children = elements.map { capture($0, depth: depth + 1) }
    }
    let enabled = (attribute(element, kAXEnabledAttribute) as? NSNumber)?.boolValue
    return Node(
        role: stringAttribute(element, kAXRoleAttribute) ?? "unknown",
        subrole: stringAttribute(element, kAXSubroleAttribute),
        title: stringAttribute(element, kAXTitleAttribute),
        value: stringAttribute(element, kAXValueAttribute),
        help: stringAttribute(element, kAXHelpAttribute),
        enabled: enabled,
        children: children
    )
}

guard CommandLine.arguments.count == 2,
      let pid = Int32(CommandLine.arguments[1]),
      pid > 0 else {
    fputs("usage: swift scripts/capture_macos_ax.swift <pid>\n", stderr)
    exit(2)
}
guard AXIsProcessTrusted() else {
    fputs("macOS Accessibility permission is required for this process\n", stderr)
    exit(2)
}
let application = AXUIElementCreateApplication(pid)
guard let windows = attribute(application, kAXWindowsAttribute) as? [AXUIElement],
      !windows.isEmpty else {
    fputs("no accessible windows for pid \(pid)\n", stderr)
    exit(1)
}
let encoder = JSONEncoder()
encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
let data = try encoder.encode(capture(application, depth: 0))
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write(Data([0x0a]))
