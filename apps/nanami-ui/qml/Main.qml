import QtQuick
import QtQuick.Controls

ApplicationWindow {
    id: window
    width: 420
    height: 260
    visible: true
    title: "Nanami"

    Rectangle {
        anchors.fill: parent
        color: "#161820"

        Column {
            anchors.centerIn: parent
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
                text: "Core connection: not connected"
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#7f8799"
                font.pixelSize: 13
                text: "UI skeleton only"
            }
        }
    }
}
