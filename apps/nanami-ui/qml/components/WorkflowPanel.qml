import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

    Button {
        text: workflowController.busy ? "Running mock workflow" : "Run mock workflow"
        enabled: !workflowController.busy
        onClicked: workflowController.startMockWorkflowStream()
    }

    Button {
        text: workflowController.busy ? "Running current project mock workflow" : "Run current project mock workflow"
        enabled: !workflowController.busy && projectController.trustStatus === "selected_trusted"
        onClicked: workflowController.startCurrentProjectMockWorkflowStream()
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Workflow ID: " + (workflowController.workflowId.length > 0 ? workflowController.workflowId : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Workflow Status: " + (workflowController.workflowStatus.length > 0 ? workflowController.workflowStatus : "none")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Project Path: " + (workflowController.projectPath.length > 0 ? workflowController.projectPath : "none")
        wrapMode: Text.Wrap
    }

    TextArea {
        width: parent.width
        height: 100
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.stepText
        placeholderText: "Steps will appear here"
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 12
        text: "Test Result"
    }

    TextArea {
        width: parent.width
        height: 70
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.testResultText
        placeholderText: "Test Result will appear here"
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 12
        text: "Patch Proposal"
    }

    TextArea {
        width: parent.width
        height: 110
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.patchText
        placeholderText: "Patch Proposal will appear here"
    }

    Button {
        text: workflowController.busy ? "Requesting mock apply patch" : "Request mock apply patch"
        enabled: !workflowController.busy
        onClicked: workflowController.requestMockApplyPatch()
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: "Apply Patch Status: " + (workflowController.applyPatchStatus.length > 0 ? workflowController.applyPatchStatus : "none")
    }

    TextArea {
        width: parent.width
        height: 70
        readOnly: true
        wrapMode: TextArea.Wrap
        text: workflowController.applyPatchText
        placeholderText: "Mock apply patch result will appear here"
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
