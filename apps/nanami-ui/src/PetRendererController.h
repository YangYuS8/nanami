#pragma once

#include <QObject>
#include <QString>

class PetRendererController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString rendererName READ rendererName NOTIFY rendererChanged)
    Q_PROPERTY(QString rendererStatus READ rendererStatus NOTIFY rendererChanged)
    Q_PROPERTY(QString rendererBackend READ rendererBackend NOTIFY rendererChanged)
    Q_PROPERTY(QString rendererAvailability READ rendererAvailability NOTIFY rendererChanged)
    Q_PROPERTY(QString modelPath READ modelPath NOTIFY rendererChanged)
    Q_PROPERTY(bool modelLoaded READ modelLoaded NOTIFY rendererChanged)
    Q_PROPERTY(QString lastRendererError READ lastRendererError NOTIFY rendererChanged)
    Q_PROPERTY(QString currentState READ currentState NOTIFY rendererChanged)
    Q_PROPERTY(QString currentEmotion READ currentEmotion NOTIFY rendererChanged)

public:
    explicit PetRendererController(QObject *parent = nullptr);

    QString rendererName() const;
    QString rendererStatus() const;
    QString rendererBackend() const;
    QString rendererAvailability() const;
    QString modelPath() const;
    bool modelLoaded() const;
    QString lastRendererError() const;
    QString currentState() const;
    QString currentEmotion() const;

    Q_INVOKABLE void setPersonaState(const QString &state, const QString &emotion);
    Q_INVOKABLE void resetRenderer();
    Q_INVOKABLE void selectPlaceholderRenderer();
    Q_INVOKABLE void selectLive2DRenderer();
    Q_INVOKABLE void setModelPath(const QString &path);
    Q_INVOKABLE void loadModel();
    Q_INVOKABLE void unloadModel();

signals:
    void rendererChanged();

private:
    void applyPlaceholderBackend();
    void applyLive2DBackendIntent();
    void clearRendererError();

    QString m_rendererName;
    QString m_rendererStatus;
    QString m_rendererBackend;
    QString m_rendererAvailability;
    QString m_modelPath;
    bool m_modelLoaded = false;
    QString m_lastRendererError;
    QString m_currentState;
    QString m_currentEmotion;
};
