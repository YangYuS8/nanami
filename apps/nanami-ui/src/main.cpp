#include "HealthController.h"

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);

    HealthController healthController;
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("healthController", &healthController);
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
    engine.loadFromModule("Nanami", "Main");
    healthController.refresh();

    return app.exec();
}
