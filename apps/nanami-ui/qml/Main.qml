import QtQuick
import QtQuick.Controls
import "components"

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
                visible: false
            }

            StatusPanel {}
            PetPanel {}
            ChatPanel {}
            TaskPanel {}
            PermissionPanel {}
            SandboxPanel {}
            ProjectPanel {}
            WorkflowPanel {}
        }
    }
}
