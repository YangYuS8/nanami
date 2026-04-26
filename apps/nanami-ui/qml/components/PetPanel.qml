import QtQuick
import QtQuick.Controls

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
            text: qsTr("Placeholder Pet View")
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

        Text { color: "#aeb4c6"; text: qsTr("Renderer: ") + petRendererController.rendererName }
        Text { color: "#aeb4c6"; text: qsTr("Renderer Status: ") + petRendererController.rendererStatus }
        Text { color: "#aeb4c6"; text: qsTr("State: ") + (petRendererController.currentState.length > 0 ? petRendererController.currentState : qsTr("none")) }
        Text { color: "#aeb4c6"; text: qsTr("Emotion: ") + (petRendererController.currentEmotion.length > 0 ? petRendererController.currentEmotion : qsTr("none")) }
        Text { color: "#aeb4c6"; text: qsTr("Source: ") + (personaController.source.length > 0 ? personaController.source : qsTr("none")) }

        Text {
            color: "#aeb4c6"
            text: personaController.text.length > 0 ? personaController.text : qsTr("Persona text will appear here")
            wrapMode: Text.Wrap
        }

        Button {
            text: personaController.busy ? qsTr("Running mock persona") : qsTr("Run mock persona stream")
            enabled: !personaController.busy
            onClicked: personaController.startMockPersonaStream()
        }

        Row {
            spacing: 8

            Button {
                text: qsTr("Toggle window")
                onClicked: desktopController.toggleMainWindow()
            }

            Button {
                text: qsTr("Test notification")
                onClicked: desktopController.showNotification(qsTr("Nanami"), qsTr("Mock desktop notification"))
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
