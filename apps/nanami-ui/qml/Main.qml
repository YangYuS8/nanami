import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
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

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 24
            spacing: 12

            TabBar {
                id: navigationBar
                Layout.fillWidth: true

                TabButton {
                    text: "Companion"
                }

                TabButton {
                    text: "Activity"
                }

                TabButton {
                    text: "Safety"
                }

                TabButton {
                    text: "Project"
                }
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: navigationBar.currentIndex

                ScrollView {
                    clip: true

                    CompanionHome {}
                }

                ScrollView {
                    clip: true

                    Column {
                        width: parent.width
                        spacing: 12

                        TaskPanel {}
                        WorkflowPanel {}
                    }
                }

                ScrollView {
                    clip: true

                    Column {
                        width: parent.width
                        spacing: 12

                        PermissionPanel {}
                        SandboxPanel {}
                    }
                }

                ScrollView {
                    clip: true

                    Column {
                        width: parent.width
                        spacing: 12

                        ProjectPanel {}
                    }
                }
            }
        }
    }
}
