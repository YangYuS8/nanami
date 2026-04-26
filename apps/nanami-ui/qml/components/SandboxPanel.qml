import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

    Button {
        text: sandboxController.busy ? "Running mock sandbox" : "Run mock sandbox"
        enabled: !sandboxController.busy
        onClicked: sandboxController.startMockSandboxStream()
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Sandbox ID: " + (sandboxController.sandboxId.length > 0 ? sandboxController.sandboxId : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Sandbox Status: " + (sandboxController.sandboxStatus.length > 0 ? sandboxController.sandboxStatus : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Template: " + (sandboxController.templateId.length > 0 ? sandboxController.templateId : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Network: " + (sandboxController.networkPolicy.length > 0 ? sandboxController.networkPolicy : "none")
    }

    Text {
        width: parent.width
        color: "#7f8799"
        font.pixelSize: 12
        wrapMode: Text.Wrap
        text: "Sandbox view is visualization-only in 0.5c. Real mount/network capability must still go through PermissionManager in future phases, and permission decisions here do not trigger sandbox execution."
    }

    TextArea {
        width: parent.width
        height: 90
        readOnly: true
        wrapMode: TextArea.Wrap
        text: sandboxController.mountText
        placeholderText: "Sandbox mounts will appear here"
    }

    TextArea {
        width: parent.width
        height: 120
        readOnly: true
        wrapMode: TextArea.Wrap
        text: sandboxController.outputText
        placeholderText: "Sandbox output will appear here"
    }

    TextArea {
        width: parent.width
        height: 90
        readOnly: true
        wrapMode: TextArea.Wrap
        text: sandboxController.artifactText
        placeholderText: "Sandbox artifacts will appear here"
    }

    TextArea {
        width: parent.width
        height: 90
        readOnly: true
        wrapMode: TextArea.Wrap
        text: permissionController.auditText
        placeholderText: "Related permission audit records will appear here"
    }

    Text {
        width: parent.width
        color: "#ff9a9a"
        font.pixelSize: 13
        text: sandboxController.error
        visible: sandboxController.error.length > 0
        wrapMode: Text.Wrap
    }
}
