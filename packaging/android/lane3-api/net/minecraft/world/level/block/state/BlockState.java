// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import net.minecraft.world.level.block.Block;

/** A block state as Horizon writes it: {@code minecraft:wheat[age=7]}. */
public class BlockState {
    private final String state;

    private BlockState(String state) {
        this.state = state;
    }

    public static BlockState of(String state) {
        return new BlockState(state == null || state.isEmpty() ? "minecraft:air" : state);
    }

    public Block getBlock() {
        int bracket = state.indexOf('[');
        return Block.byId(bracket < 0 ? state : state.substring(0, bracket));
    }

    public boolean is(Block block) {
        return getBlock().equals(block);
    }

    public boolean isAir() {
        String id = getBlock().id();
        return id.equals("minecraft:air") || id.equals("minecraft:cave_air") || id.equals("minecraft:void_air");
    }

    /** A property's value as text (`age` → `7`), or null. */
    public String horizonProperty(String name) {
        int bracket = state.indexOf('[');
        if (bracket < 0) return null;
        for (String part : state.substring(bracket + 1, state.length() - 1).split(",")) {
            int eq = part.indexOf('=');
            if (eq > 0 && part.substring(0, eq).equals(name)) return part.substring(eq + 1);
        }
        return null;
    }

    public String horizonState() {
        return state;
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof BlockState s && s.state.equals(state);
    }

    @Override
    public int hashCode() {
        return state.hashCode();
    }

    @Override
    public String toString() {
        return "Block{" + state + "}";
    }
}
