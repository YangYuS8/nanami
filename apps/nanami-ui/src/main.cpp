#include "ChatController.h"
#include "DesktopController.h"
#include "PersonaController.h"
#include "PermissionController.h"
#include "SandboxController.h"
#include "StatusController.h"
#include "TaskController.h"

#include <QApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickWindow>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);

    ChatController chatController;
    PersonaController personaController;
    DesktopController desktopController(&personaController);
    PermissionController permissionController;
    SandboxController sandboxController;
    StatusController statusController;
    TaskController taskController;
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("chatController", &chatController);
    engine.rootContext()->setContextProperty("desktopController", &desktopController);
    engine.rootContext()->setContextProperty("personaController", &personaController);
    engine.rootContext()->setContextProperty("permissionController", &permissionController);
    engine.rootContext()->setContextProperty("sandboxController", &sandboxController);
    engine.rootContext()->setContextProperty("statusController", &statusController);
    engine.rootContext()->setContextProperty("taskController", &taskController);
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
    engine.loadFromModule("Nanami", "Main");
    if (!engine.rootObjects().isEmpty()) {
        desktopController.attachWindow(
            qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst()));
    }
    statusController.refresh();

    return app.exec();
}
