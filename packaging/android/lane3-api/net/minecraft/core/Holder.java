// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

import java.util.Optional;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

/** A registry entry that may be bound to its value later. */
public interface Holder<T> {
    T value();

    boolean isBound();

    boolean is(ResourceLocation id);

    boolean is(ResourceKey<T> key);

    default boolean is(Holder<T> other) {
        Optional<ResourceKey<T>> mine = unwrapKey();
        Optional<ResourceKey<T>> theirs = other.unwrapKey();
        return mine.isPresent() ? mine.equals(theirs) : value() == other.value();
    }

    Optional<ResourceKey<T>> unwrapKey();

    default String getRegisteredName() {
        return unwrapKey().map(k -> k.location().toString()).orElse("[unregistered]");
    }

    static <T> Holder<T> direct(T value) {
        return new Holder<>() {
            public T value() {
                return value;
            }

            public boolean isBound() {
                return true;
            }

            public boolean is(ResourceLocation id) {
                return false;
            }

            public boolean is(ResourceKey<T> key) {
                return false;
            }

            public Optional<ResourceKey<T>> unwrapKey() {
                return Optional.empty();
            }
        };
    }
}
