// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.sounds;

import net.minecraft.resources.ResourceLocation;

/** A sound by its id: Horizon plays it (`/playsound`). */
public class SoundEvent {
    private final ResourceLocation location;
    private final float range;

    private SoundEvent(ResourceLocation location, float range) {
        this.location = location;
        this.range = range;
    }

    public static SoundEvent createVariableRangeEvent(ResourceLocation location) {
        return new SoundEvent(location, 16.0F);
    }

    public static SoundEvent createFixedRangeEvent(ResourceLocation location, float range) {
        return new SoundEvent(location, range);
    }

    public ResourceLocation getLocation() {
        return location;
    }

    public float getRange(float volume) {
        return volume > 1.0F ? 16.0F * volume : range;
    }

    /** The game's {@code SoundEvents} constants are not named by their ids: none here. */
    public static SoundEvent horizonConstant(String owner, String field) {
        return null;
    }

    @Override
    public String toString() {
        return "SoundEvent{" + location + "}";
    }
}
