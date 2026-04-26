import QtQuick
import QtQuick.Controls

Column {
    width: parent.width
    spacing: 8

    Button {
        text: permissionController.busy ? qsTr("Running mock permission request") : qsTr("Run mock permission request")
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

            Text { color: "#f4f1ff"; text: qsTr("Permission Request") }
            Text { color: "#aeb4c6"; text: qsTr("Level: ") + permissionController.permissionLevel }
            Text { color: "#aeb4c6"; text: qsTr("Action: ") + permissionController.permissionAction }
            Text { color: "#aeb4c6"; text: qsTr("Target: ") + permissionController.permissionTarget; wrapMode: Text.Wrap }
            Text { color: "#aeb4c6"; text: qsTr("Reason: ") + permissionController.permissionReason; wrapMode: Text.Wrap }
            Text { color: "#aeb4c6"; text: qsTr("Scope: ") + permissionController.permissionScope }
            Text { color: "#aeb4c6"; text: qsTr("Expires: ") + permissionController.permissionExpires }

            Row {
                spacing: 8
                Button { text: qsTr("Allow once"); onClicked: permissionController.resolveAllowOnce() }
                Button { text: qsTr("Allow for task"); onClicked: permissionController.resolveAllowForTask() }
                Button { text: qsTr("Deny"); onClicked: permissionController.resolveDeny() }
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
        text: qsTr("Last decision: ") + permissionController.lastDecision
    }

    TextArea {
        width: parent.width
        height: 120
        readOnly: true
        wrapMode: TextArea.Wrap
        text: permissionController.auditText
        placeholderText: qsTr("Permission audit log summary will appear here")
    }
}
