#pragma once

#include "common.h"

enum CommandType
{
    Wait,
    Move,
};

struct WaitCommandData
{};

struct MoveCommandData
{
    int from_x, from_y;
    int to_x, to_y;
};

union Command
{
    WaitCommandData wait;
    MoveCommandData move;
};

struct IssueCommandSignal
{
    Entity subject;
    std::vector<Entity> targets;
    CommandType type;
    Command data;
};

struct CommandSignal
{    
    CommandType type;
    Command data;
};

struct CommandCompletedSignal
{};

struct CommandCancelledSignal
{};

struct CommandContext
{
    Entity subject;
    std::vector<Entity> targets;
    int cost;
    bool cancelled;
};
