// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.common;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Predicate;
import java.util.function.Supplier;

/** A mod's settings: lane 3 answers each with its default (no settings file yet). */
public class ModConfigSpec implements net.neoforged.fml.config.IConfigSpec {
    ModConfigSpec() {}

    public boolean isLoaded() {
        return true;
    }

    public static class Builder {
        private final List<String> path = new ArrayList<>();

        public Builder() {}

        public Builder comment(String comment) {
            return this;
        }

        public Builder comment(String... comment) {
            return this;
        }

        public Builder translation(String key) {
            return this;
        }

        public Builder worldRestart() {
            return this;
        }

        public Builder gameRestart() {
            return this;
        }

        public Builder push(String name) {
            path.add(name);
            return this;
        }

        public Builder push(List<String> names) {
            path.addAll(names);
            return this;
        }

        public Builder pop() {
            if (!path.isEmpty()) path.remove(path.size() - 1);
            return this;
        }

        public Builder pop(int n) {
            for (int i = 0; i < n; i++) pop();
            return this;
        }

        public <T> ConfigValue<T> define(String name, T value) {
            return new ConfigValue<>(value);
        }

        public <T> ConfigValue<T> define(String name, T value, Predicate<Object> validator) {
            return new ConfigValue<>(value);
        }

        public <T> ConfigValue<T> define(String name, Supplier<T> value, Predicate<Object> validator) {
            return new ConfigValue<>(value.get());
        }

        public BooleanValue define(String name, boolean value) {
            return new BooleanValue(value);
        }

        public IntValue defineInRange(String name, int value, int min, int max) {
            return new IntValue(value);
        }

        public LongValue defineInRange(String name, long value, long min, long max) {
            return new LongValue(value);
        }

        public DoubleValue defineInRange(String name, double value, double min, double max) {
            return new DoubleValue(value);
        }

        public <T> ConfigValue<List<? extends T>> defineList(String name, List<? extends T> value, Predicate<Object> validator) {
            return new ConfigValue<>(value);
        }

        public <T> ConfigValue<List<? extends T>> defineList(String name, List<? extends T> value, Supplier<T> newElement, Predicate<Object> validator) {
            return new ConfigValue<>(value);
        }

        public <T> ConfigValue<List<? extends T>> defineListAllowEmpty(String name, List<? extends T> value, Supplier<T> newElement, Predicate<Object> validator) {
            return new ConfigValue<>(value);
        }

        public <V extends Enum<V>> EnumValue<V> defineEnum(String name, V value) {
            return new EnumValue<>(value);
        }

        public ModConfigSpec build() {
            return new ModConfigSpec();
        }
    }

    public static class ConfigValue<T> implements Supplier<T> {
        private T value;
        private final T fallback;

        ConfigValue(T value) {
            this.value = value;
            this.fallback = value;
        }

        @Override
        public T get() {
            return value;
        }

        public T getRaw() {
            return value;
        }

        public T getDefault() {
            return fallback;
        }

        public void set(T value) {
            this.value = value;
        }

        public void save() {}

        public void clearCache() {}
    }

    public static class BooleanValue extends ConfigValue<Boolean> {
        BooleanValue(boolean value) {
            super(value);
        }

        public boolean getAsBoolean() {
            return get();
        }

        public boolean isTrue() {
            return get();
        }

        public boolean isFalse() {
            return !get();
        }
    }

    public static class IntValue extends ConfigValue<Integer> {
        IntValue(int value) {
            super(value);
        }

        public int getAsInt() {
            return get();
        }
    }

    public static class LongValue extends ConfigValue<Long> {
        LongValue(long value) {
            super(value);
        }

        public long getAsLong() {
            return get();
        }
    }

    public static class DoubleValue extends ConfigValue<Double> {
        DoubleValue(double value) {
            super(value);
        }

        public double getAsDouble() {
            return get();
        }
    }

    public static class EnumValue<T extends Enum<T>> extends ConfigValue<T> {
        EnumValue(T value) {
            super(value);
        }
    }
}
