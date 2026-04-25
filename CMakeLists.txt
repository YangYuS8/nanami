cmake_minimum_required(VERSION 3.20)

project(Nanami VERSION 0.1.0 LANGUAGES CXX)

option(NANAMI_BUILD_UI "Build the Qt/QML Nanami UI skeleton" ON)
option(NANAMI_REQUIRE_QT "Fail configuration when Qt Quick is unavailable" OFF)

if(NANAMI_BUILD_UI)
    add_subdirectory(apps/nanami-ui)
endif()
