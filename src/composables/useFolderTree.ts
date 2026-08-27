import type { BrowseItem } from "@/types/cleaner"
import type { FolderBrowseItem, FolderSortKey } from "@/composables/useFolderSessionCache"

export interface FolderTreeNode extends FolderBrowseItem {
  depth: number
  expanded: boolean
  loading: boolean
  childrenLoaded: boolean
  children: FolderTreeNode[]
  /** 正在精确统计体积（无 5G / 20 万限制） */
  exactSizing?: boolean
}

export function createTreeNode(item: BrowseItem, depth = 0): FolderTreeNode {
  return {
    ...item,
    selected: false,
    depth,
    expanded: false,
    loading: false,
    childrenLoaded: false,
    children: [],
  }
}

export function sortSiblings(
  nodes: FolderTreeNode[],
  sortBy: FolderSortKey,
  sortAsc: boolean
): FolderTreeNode[] {
  const items = [...nodes]
  const dir = sortAsc ? 1 : -1

  items.sort((a, b) => {
    if (sortBy === "type") {
      if (a.is_directory !== b.is_directory) {
        return a.is_directory ? -1 : 1
      }
    }

    let cmp = 0
    switch (sortBy) {
      case "size":
        if (!a.size_ready && b.size_ready) return 1
        if (a.size_ready && !b.size_ready) return -1
        cmp = a.size_bytes - b.size_bytes
        break
      case "time":
        cmp = a.modified.localeCompare(b.modified)
        break
      case "name":
      case "type":
        cmp = a.name.localeCompare(b.name, undefined, { sensitivity: "base" })
        break
    }
    return cmp * dir
  })

  return items
}

export interface FlatTreeRow {
  node: FolderTreeNode
  /** 每层祖先是否还有后续兄弟（决定是否画贯穿竖线） */
  ancestorLines: boolean[]
  isLastSibling: boolean
}

export function flattenVisibleTreeRows(
  nodes: FolderTreeNode[],
  sortBy: FolderSortKey,
  sortAsc: boolean
): FlatTreeRow[] {
  const result: FlatTreeRow[] = []

  const walk = (list: FolderTreeNode[], ancestorLines: boolean[]) => {
    const sorted = sortSiblings(list, sortBy, sortAsc)
    sorted.forEach((node, index) => {
      const isLastSibling = index === sorted.length - 1
      result.push({ node, ancestorLines: [...ancestorLines], isLastSibling })
      if (node.is_directory && node.expanded && node.childrenLoaded) {
        walk(node.children, [...ancestorLines, !isLastSibling])
      }
    })
  }

  walk(nodes, [])
  return result
}

export function flattenVisibleTree(
  nodes: FolderTreeNode[],
  sortBy: FolderSortKey,
  sortAsc: boolean
): FolderTreeNode[] {
  return flattenVisibleTreeRows(nodes, sortBy, sortAsc).map((row) => row.node)
}

export function collectAllTreeNodes(nodes: FolderTreeNode[]): FolderTreeNode[] {
  const result: FolderTreeNode[] = []

  const walk = (list: FolderTreeNode[]) => {
    for (const node of list) {
      result.push(node)
      if (node.children.length) walk(node.children)
    }
  }

  walk(nodes)
  return result
}

export function findTreeNode(nodes: FolderTreeNode[], path: string): FolderTreeNode | null {
  for (const node of nodes) {
    if (node.path === path) return node
    if (node.children.length) {
      const found = findTreeNode(node.children, path)
      if (found) return found
    }
  }
  return null
}

export function removeTreeNodes(nodes: FolderTreeNode[], paths: Set<string>): FolderTreeNode[] {
  return nodes
    .filter((node) => !paths.has(node.path))
    .map((node) => ({
      ...node,
      children: removeTreeNodes(node.children, paths),
    }))
}

export function mergeAnalyzedTreeNodes(
  nodes: FolderTreeNode[],
  updates: BrowseItem[]
): FolderTreeNode[] {
  const map = new Map(updates.map((i) => [i.path, i]))

  const walk = (list: FolderTreeNode[]): FolderTreeNode[] =>
    list.map((node) => {
      const updated = map.get(node.path)
      const merged = updated
        ? {
            ...node,
            ...updated,
            selected: node.selected,
            depth: node.depth,
            expanded: node.expanded,
            loading: node.loading,
            childrenLoaded: node.childrenLoaded,
            children: node.children,
            exactSizing: node.exactSizing,
          }
        : node
      return {
        ...merged,
        children: walk(merged.children),
      }
    })

  return walk(nodes)
}
