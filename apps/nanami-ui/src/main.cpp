#include "ChatController.h"
#include "PermissionController.h"
#include "SandboxController.h"
#include "StatusController.h"
#include "TaskController.h"

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);

    ChatController chatController;
    PermissionController permissionController;
    SandboxController sandboxController;
    StatusController statusController;
    TaskController taskController;
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("chatController", &chatController);
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
    statusController.refresh();

    return app.exec();
}
