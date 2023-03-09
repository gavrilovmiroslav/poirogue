#include "debug.h"

#include <sstream>

void Debug_TurnOrderSystem::activate()
{
	int line = 2;
	for (auto&& [e, ap, s] : AccessWorld_QueryAllEntitiesWith<ActionPoints, Speed>::query().each())
	{
		if (is_valid(e))
		{
			std::stringstream stream;
			
			if (AccessWorld_QueryComponent<Name>::has_component(e))
			{
				stream << std::setw(10) << AccessWorld_QueryComponent<Name>::get_component(e).name;
			}
			else
			{
				stream << std::setw(10) << "?????????";
			}
			stream << std::setw(10) << " AP " << std::setw(10) << ap.ap << " SPD " << s.speed;
			
			str({ 3, line }, stream.str(), "#ffffff"_rgb);
			line++;
		}
	}
}