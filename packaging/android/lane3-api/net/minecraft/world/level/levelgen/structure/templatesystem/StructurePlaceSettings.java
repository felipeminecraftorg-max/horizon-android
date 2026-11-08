// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.levelgen.structure.templatesystem;

import net.minecraft.world.level.block.Mirror;
import net.minecraft.world.level.block.Rotation;

/** How a template is placed: what `/place template` takes of it. */
public class StructurePlaceSettings {
    private Rotation rotation = Rotation.NONE;
    private Mirror mirror = Mirror.NONE;
    private boolean ignoreEntities;
    private net.minecraft.core.BlockPos rotationPivot = net.minecraft.core.BlockPos.ZERO;
    private net.minecraft.util.RandomSource random;

    public StructurePlaceSettings setRotation(Rotation rotation) {
        this.rotation = rotation == null ? Rotation.NONE : rotation;
        return this;
    }

    public StructurePlaceSettings setMirror(Mirror mirror) {
        this.mirror = mirror == null ? Mirror.NONE : mirror;
        return this;
    }

    public StructurePlaceSettings setIgnoreEntities(boolean ignore) {
        this.ignoreEntities = ignore;
        return this;
    }

    public StructurePlaceSettings setRotationPivot(net.minecraft.core.BlockPos pivot) {
        this.rotationPivot = pivot;
        return this;
    }

    public StructurePlaceSettings setRandom(net.minecraft.util.RandomSource random) {
        this.random = random;
        return this;
    }

    public StructurePlaceSettings setKnownShape(boolean known) {
        return this;
    }

    public StructurePlaceSettings setFinalizeEntities(boolean finalize) {
        return this;
    }

    public Rotation getRotation() {
        return rotation;
    }

    public Mirror getMirror() {
        return mirror;
    }

    public boolean isIgnoreEntities() {
        return ignoreEntities;
    }

    public net.minecraft.core.BlockPos getRotationPivot() {
        return rotationPivot;
    }
}
