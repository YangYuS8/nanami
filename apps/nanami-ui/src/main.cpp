#include "ChatController.h"
#include "DesktopController.h"
#include "PersonaController.h"
#include "PetRendererController.h"
#include "ProjectController.h"
#include "PermissionController.h"
#include "SandboxController.h"
#include "StatusController.h"
#include "TaskController.h"
#include "WorkflowController.h"

#include <QApplication>
#include <QLocale>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QObject>
#include <QQuickWindow>
#include <QTranslator>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);

    QTranslator translator;
    if (translator.load(QLocale::system(), QStringLiteral("nanami"), QStringLiteral("_"), QStringLiteral(":/i18n"))) {
        app.installTranslator(&translator);
    }

    ChatController chatController;
    PersonaController personaController;
    PetRendererController petRendererController;
    DesktopController desktopController(&personaController);
    ProjectController projectController;
    PermissionController permissionController;
    SandboxController sandboxController;
    StatusController statusController;
    TaskController taskController;
    WorkflowController workflowController;
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("chatController", &chatController);
    engine.rootContext()->setContextProperty("desktopController", &desktopController);
    engine.rootContext()->setContextProperty("personaController", &personaController);
    engine.rootContext()->setContextProperty("petRendererController", &petRendererController);
    engine.rootContext()->setContextProperty("projectController", &projectController);
    engine.rootContext()->setContextProperty("permissionController", &permissionController);
    engine.rootContext()->setContextProperty("sandboxController", &sandboxController);
    engine.rootContext()->setContextProperty("statusController", &statusController);
    engine.rootContext()->setContextProperty("taskController", &taskController);
    engine.rootContext()->setContextProperty("workflowController", &workflowController);
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
    engine.loadFromModule("Nanami", "Main");
    engine.loadFromModule("Nanami", "PetWindow");
    for (QObject *object : engine.rootObjects()) {
        auto *window = qobject_cast<QQuickWindow *>(object);
        if (window == nullptr) {
            continue;
        }

        if (window->objectName() == QStringLiteral("mainWindow")) {
            desktopController.attachWindow(window);
        } else if (window->objectName() == QStringLiteral("petWindow")) {
            desktopController.attachPetWindow(window);
        }
    }
    QObject::connect(
        &personaController,
        &PersonaController::personaChanged,
        &petRendererController,
        [&personaController, &petRendererController]() {
            petRendererController.setPersonaState(
                personaController.state(), personaController.emotion());
        });
    projectController.setPermissionController(&permissionController);
    statusController.refresh();

    return app.exec();
}
