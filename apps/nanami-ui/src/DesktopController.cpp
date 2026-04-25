#include "DesktopController.h"

#include "PersonaController.h"

#include <QAction>
#include <QGuiApplication>
#include <QIcon>
#include <QStyleHints>
#include <QWindow>

DesktopController::DesktopController(PersonaController *personaController, QObject *parent)
    : QObject(parent)
    , m_personaController(personaController)
{
    setupTray();
}

void DesktopController::attachWindow(QWindow *window)
{
    m_window = window;
}

void DesktopController::showMainWindow()
{
    if (!hasWindow()) {
        return;
    }

    m_window->show();
    m_window->raise();
    m_window->requestActivate();
}

void DesktopController::hideMainWindow()
{
    if (!hasWindow()) {
        return;
    }

    m_window->hide();
}

void DesktopController::toggleMainWindow()
{
    if (!hasWindow()) {
        return;
    }

    if (m_window->isVisible()) {
        hideMainWindow();
    } else {
        showMainWindow();
    }
}

void DesktopController::showNotification(const QString &title, const QString &message)
{
    if (!m_trayIcon.isVisible()) {
        return;
    }

    m_trayIcon.showMessage(title, message, QSystemTrayIcon::Information, 3000);
}

void DesktopController::setupTray()
{
    if (!QSystemTrayIcon::isSystemTrayAvailable()) {
        return;
    }

    m_trayIcon.setToolTip(QStringLiteral("Nanami"));
    m_trayIcon.setIcon(QIcon());

    auto *toggleAction = m_trayMenu.addAction(QStringLiteral("Show/Hide Nanami"));
    connect(toggleAction, &QAction::triggered, this, &DesktopController::toggleMainWindow);

    auto *mockPersonaAction = m_trayMenu.addAction(QStringLiteral("Run mock persona stream"));
    connect(mockPersonaAction, &QAction::triggered, this, [this]() {
        if (m_personaController) {
            m_personaController->startMockPersonaStream();
        }
    });

    m_trayMenu.addSeparator();

    auto *quitAction = m_trayMenu.addAction(QStringLiteral("Quit"));
    connect(quitAction, &QAction::triggered, qApp, &QGuiApplication::quit);

    m_trayIcon.setContextMenu(&m_trayMenu);
    m_trayIcon.show();
}

bool DesktopController::hasWindow() const
{
    return !m_window.isNull();
}
