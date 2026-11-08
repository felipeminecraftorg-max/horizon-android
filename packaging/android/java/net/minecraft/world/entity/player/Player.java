// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.entity.player;

import net.minecraft.world.entity.LivingEntity;

public abstract class Player extends LivingEntity {
    public boolean isCreative() {
        return dev.horizon.android.lane3.Native.creative();
    }
}
