import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

    Button {
        text: taskController.busy ? qsTr("Running mock task") : qsTr("Run mock task")
        enabled: !taskController.busy
        onClicked: taskController.startMockTaskStream()
    }

    Row {
        width: parent.width
        spacing: 8

        TextField {
            id: taskInput
            width: parent.width - runTaskButton.width - parent.spacing
            enabled: !taskController.busy
            placeholderText: qsTr("OpenClaw task prompt")
            onAccepted: runTaskButton.clicked()
        }

        Button {
            id: runTaskButton
            text: taskController.busy ? qsTr("Running OpenClaw task") : qsTr("Run OpenClaw task")
            enabled: !taskController.busy && taskInput.text.trim().length > 0
            onClicked: {
                taskController.startOpenClawTaskStream(taskInput.text)
                taskInput.text = ""
            }
        }
    }

    TextArea {
        width: parent.width
        height: 180
        readOnly: true
        wrapMode: TextArea.Wrap
        text: taskController.taskTimelineText
        placeholderText: qsTr("Task timeline will appear here")
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Current Task ID: ") + (taskController.currentTaskId.length > 0 ? taskController.currentTaskId : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Current Task Title: ") + (taskController.currentTaskTitle.length > 0 ? taskController.currentTaskTitle : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Current Task Status: ") + (taskController.currentTaskStatus.length > 0 ? taskController.currentTaskStatus : qsTr("none"))
    }

    Text {
        width: parent.width
        color: "#aeb4c6"
        font.pixelSize: 13
        text: qsTr("Tool Count: ") + taskController.toolCount
    }

    Text {
        width: parent.width
        color: "#ff9a9a"
        font.pixelSize: 13
        text: taskController.error
        visible: taskController.error.length > 0
        wrapMode: Text.Wrap
    }
}
