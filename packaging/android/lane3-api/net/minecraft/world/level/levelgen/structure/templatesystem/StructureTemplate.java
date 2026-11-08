// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.levelgen.structure.templatesystem;

import dev.horizon.android.lane3.Native;
import net.minecraft.core.BlockPos;

/** A saved structure by its id: Horizon places it, as `/place template` does. */
public class StructureTemplate {
    private final net.minecraft.resources.ResourceLocation id;

    public StructureTemplate(net.minecraft.resources.ResourceLocation id) {
        this.id = id;
    }

    /** Places the template with its corner at {@code pos}, in the acting player's level. */
    public boolean placeInWorld(
            net.minecraft.world.level.ServerLevelAccessor level,
            BlockPos pos,
            BlockPos pivot,
            StructurePlaceSettings settings,
            net.minecraft.util.RandomSource random,
            int flags) {
        if (id == null || pos == null) return false;
        StructurePlaceSettings s = settings == null ? new StructurePlaceSettings() : settings;
        return Native.command("place template " + id + " " + pos.getX() + " " + pos.getY() + " " + pos.getZ() + " "
                + s.getRotation().getSerializedName() + " " + s.getMirror().getSerializedName());
    }

    public net.minecraft.core.Vec3i getSize() {
        return net.minecraft.core.Vec3i.ZERO;
    }
}
