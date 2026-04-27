import QtQuick
import QtQuick.Controls

Window {
    id: petWindow
    objectName: "petWindow"
    width: 180
    height: 220
    visible: true
    color: "transparent"
    flags: Qt.Window | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
    title: qsTr("Nanami Pet")

    Rectangle {
        id: shell
        anchors.fill: parent
        radius: 24
        color: "#20242fcc"
        border.color: "#514f7a"
        border.width: 1

        MouseArea {
            id: dragArea
            anchors.fill: parent
            property real lastX: 0
            property real lastY: 0

            onPressed: function(mouse) {
                lastX = mouse.x
                lastY = mouse.y
            }

            onPositionChanged: function(mouse) {
                if (!(mouse.buttons & Qt.LeftButton))
                    return

                petWindow.x += mouse.x - lastX
                petWindow.y += mouse.y - lastY
            }
        }

        Column {
            anchors.centerIn: parent
            width: parent.width - 28
            spacing: 10

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#f4f1ff"
                font.pixelSize: 14
                font.bold: true
                text: qsTr("Nanami")
            }

            Rectangle {
                width: 108
                height: 108
                radius: 54
                color: "#7c5cff"
                anchors.horizontalCenter: parent.horizontalCenter

                Text {
                    anchors.centerIn: parent
                    text: petRendererController.currentEmotion === "happy" ? "^_^"
                          : petRendererController.currentState === "thinking" ? "..."
                          : petRendererController.currentState === "error" ? ">_<"
                          : "N"
                    color: "white"
                    font.pixelSize: 30
                    font.bold: true
                }
            }

            Text {
                width: parent.width
                color: "#d8d7ef"
                horizontalAlignment: Text.AlignHCenter
                text: personaController.text.length > 0 ? personaController.text : qsTr("Companion standby")
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
            }

            Text {
                width: parent.width
                color: "#aeb4c6"
                horizontalAlignment: Text.AlignHCenter
                text: qsTr("Emotion: ") + (petRendererController.currentEmotion.length > 0 ? petRendererController.currentEmotion : qsTr("none"))
                wrapMode: Text.Wrap
            }

            Text {
                width: parent.width
                color: "#8f96ad"
                horizontalAlignment: Text.AlignHCenter
                text: qsTr("Renderer: ") + petRendererController.rendererName
                wrapMode: Text.Wrap
            }

            Text {
                width: parent.width
                color: "#8f96ad"
                horizontalAlignment: Text.AlignHCenter
                text: qsTr("Status: ") + petRendererController.rendererStatus
                wrapMode: Text.Wrap
            }
        }
    }
}
