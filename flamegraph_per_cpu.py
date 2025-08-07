import matplotlib.pyplot as plt
import matplotlib.cm as cm
import pandas as pd
import numpy as np
import math
import json
import sys

if len(sys.argv) > 1:
    file_path = sys.argv[1]
else:
    file_path = 'shared/results.json'

if len(sys.argv) > 2:
    time_filter = sys.argv[2]
    time_filter = time_filter.split('-')
    time_filter[0] = int(time_filter[0])
    time_filter[1] = int(time_filter[1])
else:
    time_filter = None

f = open(file_path, 'r')
data = json.load(f)

# Convert to DataFrame for easier handling
time_df = pd.json_normalize(data['execution_times'], meta=["function_name", "start_time", "end_time", "duration", "inner_duration", "depth", "cpuid"])

if time_filter is not None:
    time_df = time_df.query(f'start_time >= {time_filter[0]} and start_time <= {time_filter[1]}')
    time_df['start_time'] = time_df['start_time'].apply(lambda x: x - time_filter[0])
    time_df['end_time'] = time_df['end_time'].apply(lambda x: x - time_filter[0])

time_df['depth'] = time_df['depth'].apply(lambda x: x/10)
time_df['inner_duration_perc'] = time_df['inner_duration'] * 100 / time_df['duration']

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
        if alloc_direction == 'Alloc':
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
allocations_df = pd.json_normalize(data['allocations'], meta=["alloc_type", "alloc_direction", "size", "timestamp"])

if time_filter is not None:
    allocations_df = allocations_df.query(f'timestamp >= {time_filter[0]} and timestamp <= {time_filter[1]}')
    allocations_df['timestamp'] = allocations_df['timestamp'].apply(lambda x: x - time_filter[0])

cumulative_memory,timestamps = assign_allocations(allocations_df)

texts = []

# Function to update text visibility based on zoom
def update_text_visibility(event_ax):
    # Get current x-axis limits
    x_min, x_max = event_ax.get_xlim()
    visible_range = x_max - x_min

    threshold = visible_range * 0.05

    for text, duration, start_time, end_time in texts:
        # Check if bar is visible in the current view and duration is significant
        is_visible = (start_time <= x_max and end_time >= x_min) and (duration >= threshold)
        text.set_visible(is_visible)


xdp_times_df = pd.DataFrame(data['xdp_times'], columns=['timestamp', 'text'])

if time_filter is not None:
    xdp_times_df = xdp_times_df.query(f'timestamp >= {time_filter[0]} and timestamp <= {time_filter[1]}')
    xdp_times_df['timestamp'] = xdp_times_df['timestamp'].apply(lambda x: x - time_filter[0])

ignored_cpus = []
used_cpus = sorted(list([cpu for cpu in dict.fromkeys(time_df["cpuid"]) if cpu not in ignored_cpus]))

# Create figure and axis
fig = plt.figure(figsize=((len(used_cpus) + 1) * 4, 12))
#fig = plt.figure()
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

        if row["duration"] < 10000:
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

    for i, row in xdp_times_df.iterrows():
        ax.axvline(x=row['timestamp'], linestyle="--", linewidth=0.5, color='black', alpha=0.5)
        text = ax.text(
            row['timestamp'] - 10000,
            0.1,
            row['text'].replace(", ", "\n"),
            ha="center",
            color="black",
            alpha=0.5,
            rotation="vertical",
            size="x-small",
            visible=False
        )

        #texts.append((text, 1000000, row['timestamp'] - 500000, row['timestamp'] + 500000))

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