#pragma once

#include <QByteArray>
#include <QString>
#include <QStringList>

class SseStreamParser final
{
public:
    static QStringList extractDataFrames(QString *buffer, const QByteArray &data);
};
