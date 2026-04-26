#include "DesktopController.h"

#include "PersonaController.h"

#include <QAction>
#include <QBrush>
#include <QGuiApplication>
#include <QIcon>
#include <QPainter>
#include <QPen>
#include <QPixmap>
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

    QPixmap trayPixmap(32, 32);
    trayPixmap.fill(Qt::transparent);
    {
        QPainter painter(&trayPixmap);
        painter.setRenderHint(QPainter::Antialiasing, true);
        painter.setPen(Qt::NoPen);
        painter.setBrush(QBrush(QColor("#7c5cff")));
        painter.drawEllipse(2, 2, 28, 28);
        painter.setPen(QPen(Qt::white));
        auto font = painter.font();
        font.setBold(true);
        font.setPixelSize(16);
        painter.setFont(font);
        painter.drawText(trayPixmap.rect(), Qt::AlignCenter, QStringLiteral("N"));
    }

    m_trayIcon.setToolTip(tr("Nanami"));
    m_trayIcon.setIcon(QIcon(trayPixmap));

    auto *toggleAction = m_trayMenu.addAction(tr("Show/Hide Nanami"));
    connect(toggleAction, &QAction::triggered, this, &DesktopController::toggleMainWindow);

    auto *mockPersonaAction = m_trayMenu.addAction(tr("Run mock persona stream"));
    connect(mockPersonaAction, &QAction::triggered, this, [this]() {
        if (m_personaController) {
            m_personaController->startMockPersonaStream();
        }
    });

    m_trayMenu.addSeparator();

    auto *quitAction = m_trayMenu.addAction(tr("Quit"));
    connect(quitAction, &QAction::triggered, qApp, &QGuiApplication::quit);

    m_trayIcon.setContextMenu(&m_trayMenu);
    m_trayIcon.show();
}

bool DesktopController::hasWindow() const
{
    return !m_window.isNull();
}
