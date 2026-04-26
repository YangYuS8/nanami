import QtQuick

Column {
    width: parent.width
    spacing: 4

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
}
