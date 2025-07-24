import matplotlib.pyplot as plt
import matplotlib.cm as cm
import pandas as pd
import numpy as np
import math
import json

f = open('shared/execution_times.json', 'r')
data = json.load(f)

# Convert to DataFrame for easier handling
time_df = pd.json_normalize(data, meta=["function_name", "start_time", "end_time", "duration", "inner_duration", "depth", "cpuid"])
time_df['depth'] = time_df['depth'].apply(lambda x: x/10)
time_df['inner_duration_perc'] = time_df['inner_duration'] * 100 / time_df['duration']

f = open('shared/allocations.json', 'r')
data = json.load(f)

def assign_allocations(df):
    memory_usage = {'kmalloc': 0, 'kmem_cache': 0, 'total': 0}
    cumulative_memory = {'kmalloc': [0], 'kmem_cache': [0], 'total': [0]}
    timestamps = {'kmalloc': [0], 'kmem_cache': [0], 'total': [0]}

    # Sort by timestamp to ensure chronological order
    df = df.sort_values('timestamp').reset_index(drop=True)

    for _, row in df.iterrows():
        alloc_type = row['alloc_type']
        alloc_direction = row['alloc_direction']
        size = row['size']

        # Update memory usage for the specific memory_type and total
        if alloc_direction == 'Malloc':
            memory_usage[alloc_type] += size
            memory_usage['total'] += size
        elif alloc_direction == 'Free':
            memory_usage[alloc_type] -= size
            memory_usage['total'] -= size

        # Append to respective lists
        # For kmem and kmem_cache, append only if the row matches the memory_type
        if alloc_type == 'kmalloc':
            cumulative_memory['kmalloc'].append(memory_usage['kmalloc'])
            timestamps['kmalloc'].append(row['timestamp'])
        elif alloc_type == 'kmem_cache':
            cumulative_memory['kmem_cache'].append(memory_usage['kmem_cache'])
            timestamps['kmem_cache'].append(row['timestamp'])

        # Always append to total
        cumulative_memory['total'].append(memory_usage['total'])
        timestamps['total'].append(row['timestamp'])

    return cumulative_memory, timestamps

# Convert to DataFrame for easier handling
allocations_df = pd.json_normalize(data, meta=["alloc_type", "alloc_direction", "size", "timestamp"])
cumulative_memory,timestamps = assign_allocations(allocations_df)

f = open('shared/xdp_times.json', 'r')
xdp_times = json.load(f)

texts = []

# Function to update text visibility based on zoom
def update_text_visibility(event_ax):
    # Get current x-axis limits
    x_min, x_max = event_ax.get_xlim()
    visible_range = x_max - x_min

    threshold = visible_range * 0.0075

    for text, duration, start_time, end_time in texts:
        # Check if bar is visible in the current view and duration is significant
        is_visible = (start_time <= x_max and end_time >= x_min) and (duration >= threshold)
        text.set_visible(is_visible)

used_cpus = sorted(list(dict.fromkeys(time_df["cpuid"])))

# Create figure and axis
fig = plt.figure(figsize=((len(used_cpus) + 1) * 4, 12))
ax_memory = fig.add_subplot(len(used_cpus) + 1, 1, len(used_cpus) + 1)
first_ax = None

# Assign colors to functions (optional: use a colormap or hash function names)
norm = plt.Normalize(0.0, 100.0)
cmap = cm.autumn.reversed()
#cmap = cm.rainbow
plt.subplots_adjust(hspace=0.25)
for index, cpuid in enumerate(used_cpus):
    ax = fig.add_subplot(len(used_cpus) + 1, 1, 1 + index, sharex=ax_memory, sharey=first_ax)

    if first_ax is None:
        first_ax = ax

    # Plot each function call as a horizontal bar
    for i, row in time_df.sort_values(by=['function_name']).iterrows():
        if row['cpuid'] != cpuid:
            continue

        color = cmap(norm(row['inner_duration_perc']))
        ax.barh(
            row["depth"],
            row["duration"],
            left=row["start_time"],
            height=0.1,
            color=color,
            edgecolor=(0.0, 0.0, 0.0, 0.5),
            label=row["function_name"]
        )

    ax.set_title(f"CPU {cpuid}")
    ax.set_xlim(left=0)
    ax.set_ylabel("Call Stack Depth")
    ax.set_yticklabels([])

    # Optional: Add function names as text labels on bars
    for i, row in time_df.iterrows():
        if row['cpuid'] != cpuid:
            continue

        if row["duration"] < 50000:
            text = ax.text(
                row["start_time"] + row["duration"] / 2,
                row["depth"],
                row["function_name"],
                ha="center",
                va="center",
                color="black",
                rotation="vertical",
                size="small",
                visible=False
            )
        else:
            text = ax.text(
                row["start_time"] + row["duration"] / 2,
                row["depth"],
                row["function_name"],
                ha="center",
                va="center",
                color="black",
                size="small",
                visible=False
            )

        texts.append((text, row['duration'], row['start_time'], row['end_time']))

    ax.callbacks.connect('xlim_changed', update_text_visibility)
    update_text_visibility(ax)

    for xdp_time in xdp_times:
        ax.axvline(x=xdp_time[0], linestyle="--", linewidth=0.5, color='black', alpha=0.5)
    # Remove duplicate labels in legend
    #handles, labels = ax.get_legend_handles_labels()
    #by_label = dict(zip(labels, handles))
    #ax.legend(by_label.values(), by_label.keys(), loc="upper right", bbox_to_anchor=(1.145, 1.025), framealpha=1.0, edgecolor="white")

"""
sm = cm.ScalarMappable(cmap=cmap, norm=norm)
cbar = plt.colorbar(sm, label='Inner Duration (%)')
cbar.set_ticks([0, 25, 50, 75, 100])
"""
ax_memory.yaxis.tick_right()
ax_memory.set_xlabel("Time (ns)")
ax_memory.set_ylabel("Memory usage (Bytes)")

ax_memory.step(timestamps['kmalloc'], cumulative_memory['kmalloc'], where='post', marker='o', linestyle='--', color='red', alpha=0.8, label='kmalloc')
ax_memory.step(timestamps['kmem_cache'], cumulative_memory['kmem_cache'], where='post', marker='o', linestyle='--', color='blue', alpha=0.8, label='kmem_cache')
ax_memory.step(timestamps['total'], cumulative_memory['total'], where='post', marker='o', linestyle='-', color='black', alpha=1, label='Total Memory')
ax_memory.grid(True, linestyle='--', alpha=0.7)

handles, labels = ax_memory.get_legend_handles_labels()
by_label = dict(zip(labels, handles))
ax_memory.legend(by_label.values(), by_label.keys(), loc="upper right")

plt.tight_layout()
plt.show()