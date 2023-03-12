#pragma once

#include "common.h"

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
