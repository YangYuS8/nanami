import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

    Button {
        text: projectController.busy ? "Loading mock project" : "Load mock project"
        enabled: !projectController.busy
        onClicked: projectController.loadMockProject()
    }

    Button {
        text: projectController.busy ? "Selecting project folder" : "Select project folder"
        enabled: !projectController.busy
        onClicked: projectController.selectProjectFolder()
    }

    Button {
        text: projectController.busy ? "Trusting selected project" : "Trust selected project"
        enabled: !projectController.busy && projectController.trustStatus === "selected_untrusted"
        onClicked: projectController.trustSelectedProject()
    }

    Button {
        text: projectController.busy ? "Loading project structure" : "Load project structure"
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.loadProjectStructure()
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Project ID: " + (projectController.projectId.length > 0 ? projectController.projectId : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Display Name: " + (projectController.displayName.length > 0 ? projectController.displayName : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Project Path: " + (projectController.projectPath.length > 0 ? projectController.projectPath : "none")
        wrapMode: Text.Wrap
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Project Kind: " + (projectController.projectKind.length > 0 ? projectController.projectKind : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Trust Status: " + (projectController.trustStatus.length > 0 ? projectController.trustStatus : "none")
    }

    Text {
        width: parent.width
        color: "#ff9a9a"
        font.pixelSize: 13
        text: projectController.error
        visible: projectController.error.length > 0
        wrapMode: Text.Wrap
    }

    TextArea {
        width: parent.width
        height: 100
        readOnly: true
        wrapMode: TextArea.Wrap
        text: projectController.projectStructureText
        placeholderText: "Shallow project structure summary will appear here"
    }

    Button {
        text: projectController.busy ? "Requesting manifest preview permission" : "Request manifest preview permission"
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.requestManifestPreviewPermission()
    }

    Button {
        text: projectController.busy ? "Loading manifest preview" : "Load manifest preview"
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.loadManifestPreview()
    }

    Button {
        text: projectController.busy ? "Loading manifest summary" : "Load manifest summary"
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.loadManifestSummary()
    }

    TextArea {
        width: parent.width
        height: 140
        readOnly: true
        wrapMode: TextArea.Wrap
        text: projectController.manifestPreviewText
        placeholderText: "Manifest preview will appear here after explicit permission approval"
    }

    TextArea {
        width: parent.width
        height: 120
        readOnly: true
        wrapMode: TextArea.Wrap
        text: projectController.manifestSummaryText
        placeholderText: "Manifest summary will appear here after explicit permission approval"
    }
}
