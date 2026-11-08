// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.item;

import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceLocation;

public final class ItemStack {
    public static final ItemStack EMPTY = new ItemStack("minecraft:air", 0, 0);
    private final Item item;
    private int count;
    private final int damage;

    private ItemStack(String id, int count, int damage) {
        this.item = BuiltInRegistries.ITEM.get(ResourceLocation.parse(id));
        this.count = count;
        this.damage = damage;
    }

    /** Horizon's `id count damage`, or empty. */
    public static ItemStack horizon(String held) {
        if (held == null || held.isEmpty()) return EMPTY;
        String[] p = held.split(" ");
        try {
            return new ItemStack(p[0], p.length > 1 ? Integer.parseInt(p[1]) : 1, p.length > 2 ? Integer.parseInt(p[2]) : 0);
        } catch (NumberFormatException e) {
            return EMPTY;
        }
    }

    public boolean isEmpty() {
        return count <= 0 || item.id().equals("minecraft:air");
    }

    public Item getItem() {
        return isEmpty() ? BuiltInRegistries.ITEM.get(ResourceLocation.parse("minecraft:air")) : item;
    }

    public boolean is(Item other) {
        return getItem().equals(other);
    }

    public int getCount() {
        return isEmpty() ? 0 : count;
    }

    public int getDamageValue() {
        return damage;
    }

    public boolean isDamaged() {
        return damage > 0;
    }

    @Override
    public String toString() {
        return getCount() + " " + getItem();
    }
}
