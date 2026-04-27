#pragma once

#include <QString>

class Live2DRendererAdapter
{
public:
    static bool isBuiltWithLive2DSupport();
    static bool isRuntimeAvailable();
    static QString unavailableReason();
};
