import QtQuick
import QtQuick.Controls

Rectangle {
    width: parent.width
    color: "#20242f"
    radius: 12
    border.color: "#3a4152"
    border.width: 1
    implicitHeight: personaColumn.implicitHeight + 24

    function backendLabel(value) {
        if (value === "live2d")
            return qsTr("Live2D")

        return qsTr("Placeholder")
    }

    function availabilityLabel(value) {
        if (value === "available")
            return qsTr("Available")
        if (value === "unavailable")
            return qsTr("Unavailable")

        return value
    }

    function statusLabel(value) {
        switch (value) {
        case "ready":
            return qsTr("Ready")
        case "placeholder_active":
            return qsTr("Placeholder Active")
        case "placeholder_selected":
            return qsTr("Placeholder Selected")
        case "live2d_selected":
            return qsTr("Live2D Selected")
        case "live2d_unavailable":
            return qsTr("Live2D Unavailable")
        case "live2d_ready":
            return qsTr("Live2D Ready")
        case "live2d_active":
            return qsTr("Live2D Active")
        default:
            return value
        }
    }

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
        Text { color: "#aeb4c6"; text: qsTr("Renderer Status: ") + statusLabel(petRendererController.rendererStatus) }
        Text { color: "#aeb4c6"; text: qsTr("Renderer Backend: ") + backendLabel(petRendererController.rendererBackend) }
        Text { color: "#aeb4c6"; text: qsTr("Renderer Availability: ") + availabilityLabel(petRendererController.rendererAvailability) }
        Text { color: "#aeb4c6"; text: qsTr("Model Path: ") + (petRendererController.modelPath.length > 0 ? petRendererController.modelPath : qsTr("not configured")); wrapMode: Text.Wrap }
        Text { color: "#aeb4c6"; text: qsTr("Model Loaded: ") + (petRendererController.modelLoaded ? qsTr("yes") : qsTr("no")) }
        Text { color: "#aeb4c6"; text: qsTr("State: ") + (petRendererController.currentState.length > 0 ? petRendererController.currentState : qsTr("none")) }
        Text { color: "#aeb4c6"; text: qsTr("Emotion: ") + (petRendererController.currentEmotion.length > 0 ? petRendererController.currentEmotion : qsTr("none")) }
        Text { color: "#aeb4c6"; text: qsTr("Source: ") + (personaController.source.length > 0 ? personaController.source : qsTr("none")) }

        TextField {
            id: modelPathInput
            width: parent.width
            placeholderText: qsTr("Enter Live2D model path")
            text: petRendererController.modelPath
        }

        Row {
            width: parent.width
            spacing: 8

            Button {
                text: qsTr("Set model path")
                onClicked: petRendererController.setModelPath(modelPathInput.text)
            }

            Button {
                text: qsTr("Load model")
                onClicked: petRendererController.loadModel()
            }

            Button {
                text: qsTr("Unload model")
                onClicked: petRendererController.unloadModel()
            }
        }

        Row {
            width: parent.width
            spacing: 8

            Button {
                text: qsTr("Use placeholder renderer")
                onClicked: petRendererController.selectPlaceholderRenderer()
            }

            Button {
                text: qsTr("Use Live2D renderer")
                onClicked: petRendererController.selectLive2DRenderer()
            }
        }

        Text {
            color: "#aeb4c6"
            text: personaController.text.length > 0 ? personaController.text : qsTr("Persona text will appear here")
            wrapMode: Text.Wrap
        }

        Text {
            width: parent.width
            color: "#ffb3b3"
            text: petRendererController.lastRendererError
            visible: petRendererController.lastRendererError.length > 0
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
