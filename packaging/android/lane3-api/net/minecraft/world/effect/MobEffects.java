// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.effect;

import net.minecraft.core.Holder;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

/** The game's effects, by their ids. */
public final class MobEffects {
    public static final Holder<MobEffect> MOVEMENT_SPEED = of("speed");
    public static final Holder<MobEffect> MOVEMENT_SLOWDOWN = of("slowness");
    public static final Holder<MobEffect> DIG_SPEED = of("haste");
    public static final Holder<MobEffect> DIG_SLOWDOWN = of("mining_fatigue");
    public static final Holder<MobEffect> DAMAGE_BOOST = of("strength");
    public static final Holder<MobEffect> HEAL = of("instant_health");
    public static final Holder<MobEffect> HARM = of("instant_damage");
    public static final Holder<MobEffect> JUMP = of("jump_boost");
    public static final Holder<MobEffect> CONFUSION = of("nausea");
    public static final Holder<MobEffect> REGENERATION = of("regeneration");
    public static final Holder<MobEffect> DAMAGE_RESISTANCE = of("resistance");
    public static final Holder<MobEffect> FIRE_RESISTANCE = of("fire_resistance");
    public static final Holder<MobEffect> WATER_BREATHING = of("water_breathing");
    public static final Holder<MobEffect> INVISIBILITY = of("invisibility");
    public static final Holder<MobEffect> BLINDNESS = of("blindness");
    public static final Holder<MobEffect> NIGHT_VISION = of("night_vision");
    public static final Holder<MobEffect> HUNGER = of("hunger");
    public static final Holder<MobEffect> WEAKNESS = of("weakness");
    public static final Holder<MobEffect> POISON = of("poison");
    public static final Holder<MobEffect> WITHER = of("wither");
    public static final Holder<MobEffect> HEALTH_BOOST = of("health_boost");
    public static final Holder<MobEffect> ABSORPTION = of("absorption");
    public static final Holder<MobEffect> SATURATION = of("saturation");
    public static final Holder<MobEffect> GLOWING = of("glowing");
    public static final Holder<MobEffect> LEVITATION = of("levitation");
    public static final Holder<MobEffect> LUCK = of("luck");
    public static final Holder<MobEffect> UNLUCK = of("unluck");
    public static final Holder<MobEffect> SLOW_FALLING = of("slow_falling");
    public static final Holder<MobEffect> CONDUIT_POWER = of("conduit_power");
    public static final Holder<MobEffect> DOLPHINS_GRACE = of("dolphins_grace");
    public static final Holder<MobEffect> BAD_OMEN = of("bad_omen");
    public static final Holder<MobEffect> HERO_OF_THE_VILLAGE = of("hero_of_the_village");
    public static final Holder<MobEffect> DARKNESS = of("darkness");
    public static final Holder<MobEffect> TRIAL_OMEN = of("trial_omen");
    public static final Holder<MobEffect> RAID_OMEN = of("raid_omen");
    public static final Holder<MobEffect> WIND_CHARGED = of("wind_charged");
    public static final Holder<MobEffect> WEAVING = of("weaving");
    public static final Holder<MobEffect> OOZING = of("oozing");
    public static final Holder<MobEffect> INFESTED = of("infested");

    private MobEffects() {}

    @SuppressWarnings("unchecked")
    private static Holder<MobEffect> of(String id) {
        return dev.horizon.android.lane3.Neo.named((ResourceKey<MobEffect>) (ResourceKey<?>) ResourceKey.create(Registries.MOB_EFFECT, ResourceLocation.withDefaultNamespace(id)));
    }
}
