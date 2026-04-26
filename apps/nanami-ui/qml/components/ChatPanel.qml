import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

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
}
