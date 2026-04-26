#include "SseStreamParser.h"

QStringList SseStreamParser::extractDataFrames(QString *buffer, const QByteArray &data)
{
    QStringList payloads;
    if (data.isEmpty()) {
        return payloads;
    }

    buffer->append(QString::fromUtf8(data));
    int separator = buffer->indexOf(QStringLiteral("\n\n"));
    while (separator >= 0) {
        const QString frame = buffer->left(separator).trimmed();
        buffer->remove(0, separator + 2);

        if (frame.startsWith(QStringLiteral("data:"))) {
            payloads.append(frame.mid(5).trimmed());
        }

        separator = buffer->indexOf(QStringLiteral("\n\n"));
    }

    return payloads;
}
