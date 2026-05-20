package com.rin.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.Terminal
import androidx.compose.material.icons.rounded.Warning
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Divider
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.rin.terminal.TerminalSession

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SessionDialog(
    sessions: List<TerminalSession>,
    activeIndex: Int,
    onDismiss: () -> Unit,
    onSwitchSession: (Int) -> Unit,
    onCreateSession: () -> Unit,
    onCreateRootSession: () -> Unit,
    onRemoveSession: (Int) -> Unit,
    onRenameSession: (Int, String) -> Unit
) {
    var renameIndex by remember { mutableStateOf(-1) }
    var renameText by remember { mutableStateOf("") }
    var showDeleteConfirm by remember { mutableStateOf(-1) }
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    if (renameIndex >= 0) {
        AlertDialog(
            onDismissRequest = { renameIndex = -1 },
            icon = { Icon(Icons.Rounded.Edit, contentDescription = null) },
            title = { Text("Rename Session") },
            text = {
                OutlinedTextField(
                    value = renameText,
                    onValueChange = { renameText = it },
                    label = { Text("Session Name") },
                    singleLine = true,
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier.fillMaxWidth()
                )
            },
            confirmButton = {
                Button(onClick = {
                    if (renameText.isNotBlank()) {
                        onRenameSession(renameIndex, renameText.trim())
                    }
                    renameIndex = -1
                }) {
                    Text("Save")
                }
            },
            dismissButton = {
                TextButton(onClick = { renameIndex = -1 }) {
                    Text("Cancel")
                }
            }
        )
    }

    if (showDeleteConfirm >= 0) {
        val sessionName = sessions.getOrNull(showDeleteConfirm)?.name ?: ""
        AlertDialog(
            onDismissRequest = { showDeleteConfirm = -1 },
            icon = { Icon(Icons.Rounded.Warning, contentDescription = null, tint = MaterialTheme.colorScheme.error) },
            title = { Text("Remove Session") },
            text = { Text("Are you sure you want to remove \"$sessionName\"? The shell process will be terminated.") },
            confirmButton = {
                Button(
                    onClick = {
                        onRemoveSession(showDeleteConfirm)
                        showDeleteConfirm = -1
                        if (sessions.size <= 1) {
                            onDismiss()
                        }
                    },
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.error,
                        contentColor = MaterialTheme.colorScheme.onError
                    )
                ) {
                    Text("Remove")
                }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteConfirm = -1 }) {
                    Text("Cancel")
                }
            }
        )
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        dragHandle = { BottomSheetDefaults.DragHandle() },
        containerColor = MaterialTheme.colorScheme.surface
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp)
                .padding(bottom = 32.dp)
        ) {
            Text(
                text = "Terminal Sessions",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.padding(bottom = 16.dp)
            )

            LazyColumn(
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f, fill = false),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                itemsIndexed(sessions) { index, session ->
                    val isActive = index == activeIndex
                    Surface(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(16.dp))
                            .clickable {
                                onSwitchSession(index)
                                onDismiss()
                            },
                        color = if (isActive) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant,
                        shape = RoundedCornerShape(16.dp)
                    ) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(40.dp)
                                    .clip(CircleShape)
                                    .background(if (isActive) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surface),
                                contentAlignment = Alignment.Center
                            ) {
                                Icon(
                                    imageVector = Icons.Rounded.Terminal,
                                    contentDescription = null,
                                    tint = if (isActive) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface,
                                    modifier = Modifier.size(20.dp)
                                )
                            }
                            
                            Spacer(modifier = Modifier.width(16.dp))
                            
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = session.name,
                                    fontWeight = FontWeight.SemiBold,
                                    fontSize = 16.sp,
                                    color = if (isActive) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Text(
                                    text = if (isActive) "Active Session" else "Tap to switch",
                                    fontSize = 12.sp,
                                    color = if (isActive) MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f) else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                                )
                            }
                            
                            Row(
                                horizontalArrangement = Arrangement.End,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                IconButton(
                                    onClick = {
                                        renameIndex = index
                                        renameText = session.name
                                    },
                                    modifier = Modifier.size(36.dp)
                                ) {
                                    Icon(
                                        Icons.Rounded.Edit,
                                        contentDescription = "Rename",
                                        tint = if (isActive) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant,
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                                IconButton(
                                    onClick = { showDeleteConfirm = index },
                                    modifier = Modifier.size(36.dp)
                                ) {
                                    Icon(
                                        Icons.Rounded.Delete,
                                        contentDescription = "Delete",
                                        tint = MaterialTheme.colorScheme.error,
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(24.dp))
            Divider(color = MaterialTheme.colorScheme.outlineVariant, modifier = Modifier.padding(bottom = 16.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                OutlinedButton(
                    onClick = {
                        onCreateSession()
                    },
                    modifier = Modifier.weight(1f).height(56.dp),
                    shape = RoundedCornerShape(16.dp),
                    colors = ButtonDefaults.outlinedButtonColors(
                        contentColor = MaterialTheme.colorScheme.primary
                    ),
                    contentPadding = PaddingValues(0.dp)
                ) {
                    Icon(Icons.Rounded.Add, contentDescription = null, modifier = Modifier.size(20.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Session", fontWeight = FontWeight.SemiBold)
                }

                Button(
                    onClick = {
                        onCreateRootSession()
                    },
                    modifier = Modifier.weight(1f).height(56.dp),
                    shape = RoundedCornerShape(16.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer,
                        contentColor = MaterialTheme.colorScheme.onErrorContainer
                    ),
                    contentPadding = PaddingValues(0.dp)
                ) {
                    Icon(Icons.Rounded.Terminal, contentDescription = null, modifier = Modifier.size(20.dp))
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Root", fontWeight = FontWeight.SemiBold)
                }
            }
        }
    }
}
