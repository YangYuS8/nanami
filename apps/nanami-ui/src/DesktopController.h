#pragma once

#include <QMenu>
#include <QObject>
#include <QPointer>
#include <QSystemTrayIcon>

class PersonaController;
class QWindow;

class DesktopController final : public QObject
{
    Q_OBJECT

public:
    explicit DesktopController(PersonaController *personaController, QObject *parent = nullptr);

    void attachWindow(QWindow *window);
    void attachPetWindow(QWindow *window);

    Q_INVOKABLE void showMainWindow();
    Q_INVOKABLE void hideMainWindow();
    Q_INVOKABLE void toggleMainWindow();
    Q_INVOKABLE void showPetWindow();
    Q_INVOKABLE void hidePetWindow();
    Q_INVOKABLE void togglePetWindow();
    Q_INVOKABLE void showNotification(const QString &title, const QString &message);

private:
    void setupTray();
    bool hasMainWindow() const;
    bool hasPetWindow() const;

    PersonaController *m_personaController;
    QPointer<QWindow> m_mainWindow;
    QPointer<QWindow> m_petWindow;
    QSystemTrayIcon m_trayIcon;
    QMenu m_trayMenu;
};
