// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import java.util.LinkedHashMap;
import java.util.Map;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.properties.Property;

/**
 * A block state as Horizon writes it ({@code minecraft:wheat[age=7]}): its block and the values
 * it sets. A value it doesn't set is the block's default state's.
 */
public class BlockState extends StateHolder<Block, BlockState> {
    /** The block's id, or null for the block {@code owner}'s (a state made before it had one). */
    private final String id;
    private final Block owner;
    private final Map<String, String> values;

    private BlockState(String id, Block owner, Map<String, String> values) {
        this.id = id;
        this.owner = owner;
        this.values = values;
    }

    public static BlockState of(String state) {
        if (state == null || state.isEmpty()) state = "minecraft:air";
        int bracket = state.indexOf('[');
        Map<String, String> values = new LinkedHashMap<>();
        if (bracket >= 0) {
            String inside = state.substring(bracket + 1, state.endsWith("]") ? state.length() - 1 : state.length());
            for (String part : inside.split(",")) {
                int eq = part.indexOf('=');
                if (eq > 0) values.put(part.substring(0, eq).trim(), part.substring(eq + 1).trim());
            }
        }
        return new BlockState(bracket < 0 ? state : state.substring(0, bracket), null, values);
    }

    /** The block {@code owner}'s state before it has an id (its definition's {@code any()}). */
    public static BlockState horizonOf(Block owner) {
        return new BlockState(null, owner, new LinkedHashMap<>());
    }

    private String blockId() {
        if (id != null) return id;
        String own = owner == null ? null : owner.id();
        return own == null ? "minecraft:air" : own;
    }

    public Block getBlock() {
        return owner != null ? owner : Block.byId(blockId());
    }

    public boolean is(Block block) {
        Block mine = getBlock();
        return mine == block || mine.equals(block);
    }

    public boolean isAir() {
        String id = blockId();
        return id.equals("minecraft:air") || id.equals("minecraft:cave_air") || id.equals("minecraft:void_air");
    }

    /** A property's value as text ({@code age} → {@code 7}): the default state's if this one doesn't set it. */
    public String horizonProperty(String name) {
        String v = values.get(name);
        if (v != null) return v;
        BlockState d = getBlock().horizonDefault();
        return d == null || d == this ? null : d.values.get(name);
    }

    public String horizonState() {
        String block = blockId();
        if (values.isEmpty()) return block;
        StringBuilder out = new StringBuilder(block).append('[');
        boolean first = true;
        for (Map.Entry<String, String> e : values.entrySet()) {
            if (!first) out.append(',');
            out.append(e.getKey()).append('=').append(e.getValue());
            first = false;
        }
        return out.append(']').toString();
    }

    /** Whether the block defines {@code property} (its mod's definition, or Horizon's block). */
    private boolean defines(Property<?> property) {
        return getBlock().getStateDefinition().getProperty(property.getName()) != null;
    }

    @Override
    public <T extends Comparable<T>> T getValue(Property<T> property) {
        if (property == null) throw new IllegalArgumentException("Cannot get property null");
        String text = horizonProperty(property.getName());
        if (text != null) return property.horizonValue(text);
        // A property the block defines that Horizon's state lacks: its first value, as a block
        // that registered no default state has.
        if (defines(property) && !property.getPossibleValues().isEmpty()) return property.getPossibleValues().iterator().next();
        throw new IllegalArgumentException("Cannot get property " + property + " as it does not exist in " + this);
    }

    @Override
    public <T extends Comparable<T>> boolean hasProperty(Property<T> property) {
        return property != null && (horizonProperty(property.getName()) != null || defines(property));
    }

    @Override
    public <T extends Comparable<T>, V extends T> BlockState setValue(Property<T> property, V value) {
        if (property == null || value == null) throw new IllegalArgumentException("Cannot set property " + property + " to " + value);
        Map<String, String> changed = new LinkedHashMap<>(values);
        changed.put(property.getName(), property.horizonName(value));
        return new BlockState(id, owner, changed);
    }

    public java.util.Collection<Property<?>> getProperties() {
        return getBlock().getStateDefinition().getProperties();
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof BlockState s && s.horizonState().equals(horizonState());
    }

    @Override
    public int hashCode() {
        return horizonState().hashCode();
    }

    @Override
    public String toString() {
        return "Block{" + horizonState() + "}";
    }
}
