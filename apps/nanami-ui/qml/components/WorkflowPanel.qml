import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

    Button {
        text: workflowController.busy ? qsTr("Running mock workflow") : qsTr("Run mock workflow")
        enabled: !workflowController.busy
        onClicked: workflowController.startMockWorkflowStream()
    }

    Button {
        text: workflowController.busy ? qsTr("Running current project mock workflow") : qsTr("Run current project mock workflow")
        enabled: !workflowController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: workflowController.startCurrentProjectMockWorkflowStream()
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Workflow ID: ") + (workflowController.workflowId.length > 0 ? workflowController.workflowId : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Workflow Status: ") + (workflowController.workflowStatus.length > 0 ? workflowController.workflowStatus : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Project Path: ") + (workflowController.projectPath.length > 0 ? workflowController.projectPath : qsTr("none"))
        wrapMode: Text.Wrap
    }

    TextArea {
        width: parent.width
        height: 100
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.stepText
        placeholderText: qsTr("Steps will appear here")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 12
        text: qsTr("Test Result")
    }

    TextArea {
        width: parent.width
        height: 70
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.testResultText
        placeholderText: qsTr("Test Result will appear here")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 12
        text: qsTr("Patch Proposal")
    }

    TextArea {
        width: parent.width
        height: 110
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.patchText
        placeholderText: qsTr("Patch Proposal will appear here")
    }

    Button {
        text: workflowController.busy ? qsTr("Requesting mock apply patch") : qsTr("Request mock apply patch")
        enabled: !workflowController.busy
        onClicked: workflowController.requestMockApplyPatch()
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Apply Patch Status: ") + (workflowController.applyPatchStatus.length > 0 ? workflowController.applyPatchStatus : qsTr("none"))
    }

    TextArea {
        width: parent.width
        height: 70
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.applyPatchText
        placeholderText: qsTr("Mock apply patch result will appear here")
    }

    Text {
        width: parent.width
        color: "#ff9a9a"
        font.pixelSize: 13
        text: workflowController.error
        visible: workflowController.error.length > 0
        wrapMode: Text.Wrap
    }
}
