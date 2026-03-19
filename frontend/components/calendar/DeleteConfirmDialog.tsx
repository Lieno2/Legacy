"use client"

import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { AlertCircle } from "lucide-react"

interface DeleteConfirmDialogProps {
    open: boolean
    deleting: boolean
    onConfirm: () => void
    onCancel: () => void
}

export function DeleteConfirmDialog({ open, deleting, onConfirm, onCancel }: DeleteConfirmDialogProps) {
    return (
        <Dialog open={open} onOpenChange={onCancel}>
            <DialogContent className="sm:max-w-sm">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-destructive">
                        <AlertCircle className="w-5 h-5" />
                        Delete Event
                    </DialogTitle>
                </DialogHeader>
                <p className="text-sm text-muted-foreground">
                    This will permanently delete the event and all RSVPs. This action cannot be undone.
                </p>
                <DialogFooter>
                    <Button variant="ghost" onClick={onCancel} disabled={deleting}>Cancel</Button>
                    <Button variant="destructive" onClick={onConfirm} disabled={deleting}>
                        {deleting ? "Deleting…" : "Delete"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
