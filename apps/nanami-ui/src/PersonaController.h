#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class PersonaController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString state READ state NOTIFY personaChanged)
    Q_PROPERTY(QString emotion READ emotion NOTIFY personaChanged)
    Q_PROPERTY(QString text READ text NOTIFY personaChanged)
    Q_PROPERTY(QString source READ source NOTIFY personaChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

public:
    explicit PersonaController(QObject *parent = nullptr);

    QString state() const;
    QString emotion() const;
    QString text() const;
    QString source() const;
    bool busy() const;
    QString error() const;

    Q_INVOKABLE void startMockPersonaStream();

signals:
    void personaChanged();
    void busyChanged();
    void errorChanged();

private:
    void resetState();
    void handleStreamData(const QByteArray &data);
    void handleEvent(const QJsonObject &event);
    void setBusy(bool busy);
    void setError(const QString &error);

    QNetworkAccessManager m_network;
    QString m_streamBuffer;
    QString m_state;
    QString m_emotion;
    QString m_text;
    QString m_source;
    QString m_error;
    bool m_busy = false;
};
