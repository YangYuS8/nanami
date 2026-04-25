cmake_minimum_required(VERSION 3.20)

project(nanami-ui VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Qt6 QUIET COMPONENTS Network Quick)

if(NOT Qt6_FOUND)
    if(NANAMI_REQUIRE_QT)
        find_package(Qt6 REQUIRED COMPONENTS Network Quick)
    endif()

    message(WARNING "Qt6 Quick was not found; skipping nanami-ui target")
    return()
endif()

qt_policy(SET QTP0004 NEW)
qt_standard_project_setup(REQUIRES 6.5)

qt_add_executable(nanami-ui
    src/ChatController.cpp
    src/ChatController.h
    src/StatusController.cpp
    src/StatusController.h
    src/main.cpp
)

qt_add_qml_module(nanami-ui
    URI Nanami
    VERSION 1.0
    QML_FILES
        qml/Main.qml
)

target_link_libraries(nanami-ui
    PRIVATE
        Qt6::Network
        Qt6::Quick
)
