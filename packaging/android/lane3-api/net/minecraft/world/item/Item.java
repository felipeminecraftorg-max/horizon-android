// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.item;

public class Item {
    final String id;

    public Item(String id) {
        this.id = id;
    }

    public String id() {
        return id;
    }

    /** The item {@code id} names, as the registry holds it (one object per id). */
    public static Item byId(String id) {
        return net.minecraft.core.registries.BuiltInRegistries.ITEM.get(net.minecraft.resources.ResourceLocation.parse(id));
    }

    public String getDescriptionId() {
        int colon = id.indexOf(':');
        return "item." + id.substring(0, colon) + "." + id.substring(colon + 1);
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof Item i && i.id.equals(id);
    }

    @Override
    public int hashCode() {
        return id.hashCode();
    }

    @Override
    public String toString() {
        return id;
    }
}
