// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.registries;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.function.Function;
import java.util.function.Supplier;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.item.Item;
import net.minecraft.world.level.block.Block;
import net.neoforged.bus.api.IEventBus;

/** What a mod registers in one registry: lane 3 makes each value the first time it is asked for. */
public class DeferredRegister<T> {
    private final ResourceKey<? extends Registry<T>> registryKey;
    private final String namespace;
    private final List<DeferredHolder<T, ? extends T>> entries = new ArrayList<>();

    protected DeferredRegister(ResourceKey<? extends Registry<T>> registryKey, String namespace) {
        this.registryKey = registryKey;
        this.namespace = namespace;
    }

    public static <T> DeferredRegister<T> create(ResourceKey<? extends Registry<T>> key, String namespace) {
        return new DeferredRegister<>(key, namespace);
    }

    /** For the registry itself ({@code BuiltInRegistries.BLOCK_ENTITY_TYPE}). */
    public static <T> DeferredRegister<T> create(Registry<T> registry, String namespace) {
        ResourceKey<? extends Registry<T>> key = registry == null ? null : registry.key();
        return new DeferredRegister<>(key != null ? key : ResourceKey.<T>createRegistryKey(ResourceLocation.parse("horizon:unknown")), namespace);
    }

    public static <T> DeferredRegister<T> create(ResourceLocation registry, String namespace) {
        return new DeferredRegister<>(ResourceKey.<T>createRegistryKey(registry), namespace);
    }

    public static Blocks createBlocks(String namespace) {
        return new Blocks(namespace);
    }

    public static Items createItems(String namespace) {
        return new Items(namespace);
    }

    protected <I extends T> DeferredHolder<T, I> createHolder(ResourceKey<? extends Registry<T>> registry, ResourceLocation id) {
        return DeferredHolder.create(registry, id);
    }

    public <I extends T> DeferredHolder<T, I> register(String name, Supplier<? extends I> supplier) {
        return register(name, id -> supplier.get());
    }

    public <I extends T> DeferredHolder<T, I> register(String name, Function<ResourceLocation, ? extends I> make) {
        ResourceLocation id = ResourceLocation.fromNamespaceAndPath(namespace, name);
        DeferredHolder<T, I> holder = createHolder(registryKey, id);
        entries.add(holder);
        dev.horizon.android.lane3.Neo.define(holder.getKey(), () -> make.apply(id));
        return holder;
    }

    public void register(IEventBus bus) {
        dev.horizon.android.lane3.Neo.registered(registryKey.location().toString(), entries.size());
    }

    public Collection<DeferredHolder<T, ? extends T>> getEntries() {
        return Collections.unmodifiableList(entries);
    }

    public ResourceKey<? extends Registry<T>> getRegistryKey() {
        return registryKey;
    }

    public ResourceLocation getRegistryName() {
        return registryKey.location();
    }

    public String getNamespace() {
        return namespace;
    }

    /** The blocks a mod registers. */
    @SuppressWarnings("unchecked")
    public static class Blocks extends DeferredRegister<Block> {
        protected Blocks(String namespace) {
            super((ResourceKey<? extends Registry<Block>>) (ResourceKey<?>) Registries.BLOCK, namespace);
        }

        @Override
        protected <I extends Block> DeferredHolder<Block, I> createHolder(ResourceKey<? extends Registry<Block>> registry, ResourceLocation id) {
            return DeferredBlock.createBlock(ResourceKey.create(registry, id));
        }

        @Override
        public <B extends Block> DeferredBlock<B> register(String name, Supplier<? extends B> supplier) {
            return (DeferredBlock<B>) super.register(name, supplier);
        }

        @Override
        public <B extends Block> DeferredBlock<B> register(String name, Function<ResourceLocation, ? extends B> make) {
            return (DeferredBlock<B>) super.register(name, make);
        }
    }

    /** The items a mod registers. */
    @SuppressWarnings("unchecked")
    public static class Items extends DeferredRegister<Item> {
        protected Items(String namespace) {
            super((ResourceKey<? extends Registry<Item>>) (ResourceKey<?>) Registries.ITEM, namespace);
        }

        @Override
        protected <I extends Item> DeferredHolder<Item, I> createHolder(ResourceKey<? extends Registry<Item>> registry, ResourceLocation id) {
            return DeferredItem.createItem(ResourceKey.create(registry, id));
        }

        @Override
        public <I extends Item> DeferredItem<I> register(String name, Supplier<? extends I> supplier) {
            return (DeferredItem<I>) super.register(name, supplier);
        }

        @Override
        public <I extends Item> DeferredItem<I> register(String name, Function<ResourceLocation, ? extends I> make) {
            return (DeferredItem<I>) super.register(name, make);
        }
    }
}
