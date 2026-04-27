import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 16

    Rectangle {
        width: parent.width
        color: "#1d2130"
        radius: 14
        border.color: "#31384a"
        border.width: 1
        implicitHeight: introColumn.implicitHeight + 28

        Column {
            id: introColumn
            anchors.fill: parent
            anchors.margins: 14
            spacing: 6

            Text {
                color: "#f4f1ff"
                font.pixelSize: 22
                font.bold: true
                text: qsTr("Companion Home")
            }

            Text {
                width: parent.width
                color: "#c7cbe0"
                font.pixelSize: 14
                wrapMode: Text.Wrap
                text: qsTr("Nanami is your local companion client for OpenClaw. Start a chat, check the companion state, and keep core runtime status visible without leaving the desktop view.")
            }

            Button {
                text: qsTr("Toggle pet window")
                onClicked: desktopController.togglePetWindow()
            }
        }
    }

    PetPanel {}
    ChatPanel {}

    Rectangle {
        width: parent.width
        color: "#1a1e2b"
        radius: 12
        border.color: "#2c3344"
        border.width: 1
        implicitHeight: statusColumn.implicitHeight + 24

        Column {
            id: statusColumn
            anchors.fill: parent
            anchors.margins: 12
            spacing: 10

            Text {
                color: "#d7dcf0"
                font.pixelSize: 14
                font.bold: true
                text: qsTr("Connection Status")
            }

            StatusPanel {
                width: parent.width
            }
        }
    }
}
