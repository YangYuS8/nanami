import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

    Button {
        text: projectController.busy ? qsTr("Loading mock project") : qsTr("Load mock project")
        enabled: !projectController.busy
        onClicked: projectController.loadMockProject()
    }

    Button {
        text: projectController.busy ? qsTr("Selecting project folder") : qsTr("Select project folder")
        enabled: !projectController.busy
        onClicked: projectController.selectProjectFolder()
    }

    Button {
        text: projectController.busy ? qsTr("Trusting selected project") : qsTr("Trust selected project")
        enabled: !projectController.busy && projectController.trustStatus === "selected_untrusted"
        onClicked: projectController.trustSelectedProject()
    }

    Button {
        text: projectController.busy ? qsTr("Loading project structure") : qsTr("Load project structure")
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.loadProjectStructure()
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Project ID: ") + (projectController.projectId.length > 0 ? projectController.projectId : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Display Name: ") + (projectController.displayName.length > 0 ? projectController.displayName : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Project Path: ") + (projectController.projectPath.length > 0 ? projectController.projectPath : qsTr("none"))
        wrapMode: Text.Wrap
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Project Kind: ") + (projectController.projectKind.length > 0 ? projectController.projectKind : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Trust Status: ") + (projectController.trustStatus.length > 0 ? projectController.trustStatus : qsTr("none"))
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
        placeholderText: qsTr("Shallow project structure summary will appear here")
    }

    Button {
        text: projectController.busy ? qsTr("Requesting manifest preview permission") : qsTr("Request manifest preview permission")
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.requestManifestPreviewPermission()
    }

    Button {
        text: projectController.busy ? qsTr("Loading manifest preview") : qsTr("Load manifest preview")
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.loadManifestPreview()
    }

    Button {
        text: projectController.busy ? qsTr("Loading manifest summary") : qsTr("Load manifest summary")
        enabled: !projectController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: projectController.loadManifestSummary()
    }

    TextArea {
        width: parent.width
        height: 140
        readOnly: true
        wrapMode: TextArea.Wrap
        text: projectController.manifestPreviewText
        placeholderText: qsTr("Manifest preview will appear here after explicit permission approval")
    }

    TextArea {
        width: parent.width
        height: 120
        readOnly: true
        wrapMode: TextArea.Wrap
        text: projectController.manifestSummaryText
        placeholderText: qsTr("Manifest summary will appear here after explicit permission approval")
    }
}
