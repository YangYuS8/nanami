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

            Rectangle {
                width: parent.width
                color: "#20242f"
                radius: 12
                border.color: "#3a4152"
                border.width: 1
                implicitHeight: personaColumn.implicitHeight + 24

                Column {
                    id: personaColumn
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 8

                    Text {
                        color: "#f4f1ff"
                        text: "Placeholder Pet View"
                        font.pixelSize: 16
                        font.bold: true
                    }

                    Rectangle {
                        width: 88
                        height: 88
                        radius: 44
                        color: "#7c5cff"
                        anchors.horizontalCenter: parent.horizontalCenter

                        Text {
                            anchors.centerIn: parent
                            text: petRendererController.currentEmotion === "happy" ? "^_^"
                                  : petRendererController.currentState === "thinking" ? "..."
                                  : petRendererController.currentState === "error" ? ">_<"
                                  : "N"
                            color: "white"
                            font.pixelSize: 26
                            font.bold: true
                        }
                    }

                    Text { color: "#aeb4c6"; text: "Renderer: " + petRendererController.rendererName }
                    Text { color: "#aeb4c6"; text: "Renderer Status: " + petRendererController.rendererStatus }
                    Text { color: "#aeb4c6"; text: "State: " + (petRendererController.currentState.length > 0 ? petRendererController.currentState : "none") }
                    Text { color: "#aeb4c6"; text: "Emotion: " + (petRendererController.currentEmotion.length > 0 ? petRendererController.currentEmotion : "none") }
                    Text { color: "#aeb4c6"; text: "Source: " + (personaController.source.length > 0 ? personaController.source : "none") }
                    Text {
                        color: "#aeb4c6"
                        text: personaController.text.length > 0 ? personaController.text : "Persona text will appear here"
                        wrapMode: Text.Wrap
                    }

                    Button {
                        text: personaController.busy ? "Running mock persona" : "Run mock persona stream"
                        enabled: !personaController.busy
                        onClicked: personaController.startMockPersonaStream()
                    }

                    Row {
                        spacing: 8

                        Button {
                            text: "Toggle window"
                            onClicked: desktopController.toggleMainWindow()
                        }

                        Button {
                            text: "Test notification"
                            onClicked: desktopController.showNotification("Nanami", "Mock desktop notification")
                        }
                    }

                    Text {
                        color: "#ff9a9a"
                        text: personaController.error
                        visible: personaController.error.length > 0
                        wrapMode: Text.Wrap
                    }
                }
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
                placeholderText: "Task timeline will appear here"
            }

            Text {
                width: parent.width
                color: "#aeb4c6"
                font.pixelSize: 13
                text: "Current Task ID: " + (taskController.currentTaskId.length > 0 ? taskController.currentTaskId : "none")
            }

            Text {
                width: parent.width
                color: "#aeb4c6"
                font.pixelSize: 13
                text: "Current Task Title: " + (taskController.currentTaskTitle.length > 0 ? taskController.currentTaskTitle : "none")
            }

            Text {
                width: parent.width
                color: "#aeb4c6"
                font.pixelSize: 13
                text: "Current Task Status: " + (taskController.currentTaskStatus.length > 0 ? taskController.currentTaskStatus : "none")
            }

            Text {
                width: parent.width
                color: "#aeb4c6"
                font.pixelSize: 13
                text: "Tool Count: " + taskController.toolCount
            }

            Text {
                width: parent.width
                color: "#ff9a9a"
                font.pixelSize: 13
                text: taskController.error
                visible: taskController.error.length > 0
                wrapMode: Text.Wrap
            }

            Button {
                text: permissionController.busy ? "Running mock permission request" : "Run mock permission request"
                enabled: !permissionController.busy
                onClicked: permissionController.startMockPermissionStream()
            }

            Rectangle {
                width: parent.width
                color: "#20242f"
                radius: 8
                border.color: "#3a4152"
                border.width: 1
                visible: permissionController.hasPermissionRequest
                implicitHeight: permissionColumn.implicitHeight + 24

                Column {
                    id: permissionColumn
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 8

                    Text { color: "#f4f1ff"; text: "Permission Request" }
                    Text { color: "#aeb4c6"; text: "Level: " + permissionController.permissionLevel }
                    Text { color: "#aeb4c6"; text: "Action: " + permissionController.permissionAction }
                    Text { color: "#aeb4c6"; text: "Target: " + permissionController.permissionTarget; wrapMode: Text.Wrap }
                    Text { color: "#aeb4c6"; text: "Reason: " + permissionController.permissionReason; wrapMode: Text.Wrap }
                    Text { color: "#aeb4c6"; text: "Scope: " + permissionController.permissionScope }
                    Text { color: "#aeb4c6"; text: "Expires: " + permissionController.permissionExpires }

                    Row {
                        spacing: 8
                        Button { text: "Allow once"; onClicked: permissionController.resolveAllowOnce() }
                        Button { text: "Allow for task"; onClicked: permissionController.resolveAllowForTask() }
                        Button { text: "Deny"; onClicked: permissionController.resolveDeny() }
                    }
                }
            }

            Text {
                width: parent.width
                color: "#ff9a9a"
                font.pixelSize: 13
                text: permissionController.error
                visible: permissionController.error.length > 0
                wrapMode: Text.Wrap
            }

            Text {
                width: parent.width
                color: "#aeb4c6"
                font.pixelSize: 13
                text: "Last decision: " + permissionController.lastDecision
            }

            TextArea {
                width: parent.width
                height: 120
                readOnly: true
                wrapMode: TextArea.Wrap
                text: permissionController.auditText
                placeholderText: "Permission audit log summary will appear here"
            }

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

            Button {
                text: workflowController.busy ? "Running mock workflow" : "Run mock workflow"
                enabled: !workflowController.busy
                onClicked: workflowController.startMockWorkflowStream()
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
                placeholderText: "Workflow steps will appear here"
            }

            TextArea {
                width: parent.width
                height: 70
                readOnly: true
                wrapMode: TextArea.Wrap
                text: workflowController.testResultText
                placeholderText: "Workflow test result will appear here"
            }

            TextArea {
                width: parent.width
                height: 110
                readOnly: true
                wrapMode: TextArea.Wrap
                text: workflowController.patchText
                placeholderText: "Mock patch proposal will appear here"
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
    }
}
