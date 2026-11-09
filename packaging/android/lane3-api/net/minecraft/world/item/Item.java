// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.item;

public class Item {
    String id;

    public Item(String id) {
        this.id = id;
    }

    /** A mod's item: its id is the one it is registered under (lane 3 sets it). */
    public Item(Properties properties) {}

    protected Item() {}

    /** The id lane 3 registered this item under. */
    public final void horizonId(String id) {
        if (this.id == null) this.id = id;
    }

    /** How an item is made: lane 3 keeps none of it (lane 1 carried the item's). */
    public static class Properties {
        public Properties() {}
    }

    public String id() {
        return id;
    }

    /** A constant of the game's ({@code Items.STICK}): the item of the id its field names. */
    public static Item horizonConstant(String owner, String field) {
        return owner.endsWith("/Items") ? byId("minecraft:" + field.toLowerCase(java.util.Locale.ROOT)) : null;
    }

    /** The item {@code id} names, as the registry holds it (one object per id). */
    public static Item byId(String id) {
        return net.minecraft.core.registries.BuiltInRegistries.ITEM.get(net.minecraft.resources.ResourceLocation.parse(id));
    }

    public String getDescriptionId() {
        if (id == null) return "item.unknown";
        int colon = id.indexOf(':');
        return "item." + id.substring(0, colon) + "." + id.substring(colon + 1);
    }

    @Override
    public boolean equals(Object o) {
        return o == this || (o instanceof Item i && id != null && id.equals(i.id));
    }

    @Override
    public int hashCode() {
        return id == null ? System.identityHashCode(this) : id.hashCode();
    }

    @Override
    public String toString() {
        return id;
    }
}
