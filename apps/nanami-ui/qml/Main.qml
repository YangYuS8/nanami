import QtQuick
import QtQuick.Controls

ApplicationWindow {
    id: window
    width: 720
    height: 760
    visible: true
    title: "Nanami"

    Rectangle {
        anchors.fill: parent
        color: "#161820"

        Column {
            anchors.fill: parent
            anchors.margins: 24
            spacing: 12

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#f4f1ff"
                font.pixelSize: 28
                font.bold: true
                text: "Nanami"
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#aeb4c6"
                font.pixelSize: 15
                text: "Core connection: " + statusController.coreStatus
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#aeb4c6"
                font.pixelSize: 15
                text: "OpenClaw: " + statusController.openClawStatus
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#7f8799"
                font.pixelSize: 13
                text: "Gateway URL: " + (statusController.openClawGatewayUrl.length > 0 ? statusController.openClawGatewayUrl : "not configured")
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#7f8799"
                font.pixelSize: 13
                text: statusController.openClawMessage
                visible: statusController.openClawMessage.length > 0
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#7f8799"
                font.pixelSize: 13
                text: "OpenClaw status skeleton only"
            }

            TextArea {
                id: conversation
                width: parent.width
                height: 220
                readOnly: true
                wrapMode: TextArea.Wrap
                text: chatController.conversationText
                placeholderText: "Conversation will appear here"
            }

            Text {
                width: parent.width
                color: "#ff9a9a"
                font.pixelSize: 13
                text: chatController.error
                visible: chatController.error.length > 0
                wrapMode: Text.Wrap
            }

            Row {
                width: parent.width
                spacing: 8

                TextField {
                    id: chatInput
                    width: parent.width - sendButton.width - parent.spacing
                    enabled: !chatController.busy
                    placeholderText: "Message OpenClaw through nanami-core"
                    onAccepted: sendButton.clicked()
                }

                Button {
                    id: sendButton
                    text: chatController.busy ? "Sending" : "Send"
                    enabled: !chatController.busy && chatInput.text.trim().length > 0
                    onClicked: {
                        chatController.sendMessage(chatInput.text)
                        chatInput.text = ""
                    }
                }
            }

            Button {
                text: taskController.busy ? "Running mock task" : "Run mock task"
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
                    placeholderText: "OpenClaw task prompt"
                    onAccepted: runTaskButton.clicked()
                }

                Button {
                    id: runTaskButton
                    text: taskController.busy ? "Running OpenClaw task" : "Run OpenClaw task"
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
                placeholderText: "Mock task timeline will appear here"
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
    }
}
