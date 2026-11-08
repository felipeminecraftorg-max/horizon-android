// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.registries;

import net.minecraft.resources.ResourceKey;
import net.minecraft.world.item.Item;

/** An item a mod registered. */
public class DeferredItem<T extends Item> extends DeferredHolder<Item, T> {
    protected DeferredItem(ResourceKey<Item> key) {
        super(key);
    }

    public static <T extends Item> DeferredItem<T> createItem(ResourceKey<Item> key) {
        return new DeferredItem<>(key);
    }
}
