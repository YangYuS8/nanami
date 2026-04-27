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
    m_mainWindow = window;
}

void DesktopController::attachPetWindow(QWindow *window)
{
    m_petWindow = window;
}

void DesktopController::showMainWindow()
{
    if (!hasMainWindow()) {
        return;
    }

    m_mainWindow->show();
    m_mainWindow->raise();
    m_mainWindow->requestActivate();
}

void DesktopController::hideMainWindow()
{
    if (!hasMainWindow()) {
        return;
    }

    m_mainWindow->hide();
}

void DesktopController::toggleMainWindow()
{
    if (!hasMainWindow()) {
        return;
    }

    if (m_mainWindow->isVisible()) {
        hideMainWindow();
    } else {
        showMainWindow();
    }
}

void DesktopController::showPetWindow()
{
    if (!hasPetWindow()) {
        return;
    }

    m_petWindow->show();
    m_petWindow->raise();
    m_petWindow->requestActivate();
}

void DesktopController::hidePetWindow()
{
    if (!hasPetWindow()) {
        return;
    }

    m_petWindow->hide();
}

void DesktopController::togglePetWindow()
{
    if (!hasPetWindow()) {
        return;
    }

    if (m_petWindow->isVisible()) {
        hidePetWindow();
    } else {
        showPetWindow();
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

    auto *togglePetAction = m_trayMenu.addAction(tr("Show/Hide Pet"));
    connect(togglePetAction, &QAction::triggered, this, &DesktopController::togglePetWindow);

    auto *toggleMainAction = m_trayMenu.addAction(tr("Show/Hide Main Window"));
    connect(toggleMainAction, &QAction::triggered, this, &DesktopController::toggleMainWindow);

    m_trayMenu.addSeparator();

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

bool DesktopController::hasMainWindow() const
{
    return !m_mainWindow.isNull();
}

bool DesktopController::hasPetWindow() const
{
    return !m_petWindow.isNull();
}
