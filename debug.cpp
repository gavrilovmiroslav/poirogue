#include "debug.h"

#include <sstream>

#include <yaml-cpp/yaml.h>

void Debug_TurnOrderSystem::react_to_event(KeyEvent& signal)
{
	if (signal.key == KeyCode::KEY_F1)
	{
		visible = !visible;
	}
}

void Debug_TurnOrderSystem::activate()
{
	if (visible)
	{
		int line = 2;
		//                          34567890123456789012345678901234567890
		str({ 3 , line }, "NAME       AP   SPD   HP            ", "#b0b0ff"_rgb);
		line++;
		for (auto&& [e, ap, s, h] : AccessWorld_QueryAllEntitiesWith<ActionPoints, Speed, Health>::query().each())
		{
			auto color = line % 2 == 0 ? "#e0e0ff"_rgb : "#ffffff"_rgb;
			if (is_valid(e))
			{
				if (AccessWorld_QueryComponent<Name>::has_component(e))
				{
					std::stringstream stream;
					stream << std::left << std::setw(10) << std::setfill(' ') << AccessWorld_QueryComponent<Name>::get_component(e).name;
					str({ 3, line }, stream.str(), color);
					
					stream.clear(); stream.str("");
					stream << std::left << std::setw(6) << std::setfill(' ') << ap.ap;
					str({ 14, line }, stream.str(), color);

					stream.clear(); stream.str("");
					stream << std::left << std::setw(6) << std::setfill(' ') << s.speed;
					str({ 19, line }, stream.str(), color);

					stream.clear(); stream.str("");
					stream << h.current_hp << "/" << h.max_hp;
					str({ 25, line }, stream.str(), color);
				}
				line++;
			}
		}
	}
}

void Debug_ReloadConfigSystem::react_to_event(KeyEvent& signal)
{
	if (signal.key == KeyCode::KEY_F2)
	{
		auto& colors = AccessWorld_UseUnique<Colors>::access_unique();
		auto yaml_colors = AccessYAML::load("data/lists/colors.yaml");

		colors.visible_hue = yaml_colors["visible-hue"].as<float>();
		colors.visible_sat = yaml_colors["visible-sat"].as<float>();
		colors.visible_shift_mid = yaml_colors["visible-shift-mid"].as<float>();
		colors.visible_shift_far = yaml_colors["visible-shift-far"].as<float>();
		colors.visible_shift_very_far = yaml_colors["visible-shift-very-far"].as<float>();

		colors.memory_hue = yaml_colors["memory-hue"].as<float>();
		colors.memory_sat = yaml_colors["memory-sat"].as<float>();
		colors.memory_lit = yaml_colors["memory-lit"].as<float>();

		colors.shimmer_hue = yaml_colors["shimmer-hue"].as<float>();
		colors.shimmer_stripe_strength = yaml_colors["shimmer-stripe-strength"].as<float>();
		colors.shimmer_stripe_speed = yaml_colors["shimmer-stripe-speed"].as<float>();
		colors.shimmer_stripe_width = yaml_colors["shimmer-stripe-width"].as<float>();
	}
}

void Debug_HintSystem::activate()
{
	str({ 0, 0 }, " [F1: TURN ORDER] [F2: RELOAD COLORS] [F3: NEW WORLD]", "#ff0000"_rgb);
}
