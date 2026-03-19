"use client"

import { useState, useEffect } from "react"
import { signOut } from "next-auth/react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { AlertCircle, CheckCircle, LogOut, User, Mail, Lock, Shield, ArrowLeft } from "lucide-react"
import Link from "next/link"

interface UserProfile {
    id: string
    username: string
    email: string
    perms: number
    createdAt: string
}

export default function AccountPage() {
    const [profile, setProfile] = useState<UserProfile | null>(null)
    const [loading, setLoading] = useState(true)

    // Edit form
    const [username, setUsername] = useState("")
    const [email, setEmail] = useState("")
    const [currentPassword, setCurrentPassword] = useState("")
    const [newPassword, setNewPassword] = useState("")
    const [confirmPassword, setConfirmPassword] = useState("")
    const [saving, setSaving] = useState(false)
    const [error, setError] = useState("")
    const [success, setSuccess] = useState("")

    useEffect(() => {
        fetch("/api/account")
            .then(r => r.json())
            .then(data => {
                setProfile(data)
                setUsername(data.username)
                setEmail(data.email)
                setLoading(false)
            })
            .catch(() => setLoading(false))
    }, [])

    async function handleSave(e: React.FormEvent) {
        e.preventDefault()
        setError("")
        setSuccess("")

        if (newPassword && newPassword !== confirmPassword) {
            setError("New passwords do not match")
            return
        }
        if (newPassword && newPassword.length < 8) {
            setError("New password must be at least 8 characters")
            return
        }

        setSaving(true)
        try {
            const res = await fetch("/api/account", {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    username,
                    email,
                    currentPassword: currentPassword || undefined,
                    newPassword: newPassword || undefined,
                }),
            })
            const data = await res.json()
            if (!res.ok) {
                setError(data.error || "Failed to update account")
            } else {
                setProfile(prev => prev ? { ...prev, ...data } : data)
                setSuccess("Account updated successfully")
                setCurrentPassword("")
                setNewPassword("")
                setConfirmPassword("")
            }
        } catch {
            setError("Failed to update account")
        } finally {
            setSaving(false)
        }
    }

    if (loading) {
        return (
            <div className="flex items-center justify-center min-h-screen bg-background">
                <div className="text-muted-foreground text-sm">Loading…</div>
            </div>
        )
    }

    const isAdmin = (profile?.perms ?? 0) >= 999

    return (
        <div className="min-h-screen bg-background text-foreground">
            {/* Header */}
            <div className="border-b border-border/50 px-6 py-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <Link href="/calendar">
                        <Button variant="ghost" size="icon-sm">
                            <ArrowLeft className="w-4 h-4" />
                        </Button>
                    </Link>
                    <h1 className="text-lg font-semibold">Account Settings</h1>
                </div>
                <div className="flex items-center gap-2">
                    {isAdmin && (
                        <Link href="/admin">
                            <Button variant="outline" size="sm">
                                <Shield className="w-4 h-4 mr-1.5" />
                                Admin Panel
                            </Button>
                        </Link>
                    )}
                    <Button variant="ghost" size="sm" onClick={() => signOut({ callbackUrl: "/login" })}>
                        <LogOut className="w-4 h-4 mr-1.5" />
                        Sign out
                    </Button>
                </div>
            </div>

            <div className="max-w-xl mx-auto px-6 py-10 flex flex-col gap-6">

                {/* Profile summary */}
                <Card>
                    <CardContent className="pt-6 flex items-center gap-4">
                        <div className="w-14 h-14 rounded-full bg-primary/10 flex items-center justify-center text-2xl font-bold text-primary uppercase">
                            {profile?.username?.[0] ?? "?"}
                        </div>
                        <div>
                            <div className="font-semibold text-base">{profile?.username}</div>
                            <div className="text-sm text-muted-foreground">{profile?.email}</div>
                            <div className="flex items-center gap-1.5 mt-1">
                                {isAdmin ? (
                                    <span className="text-xs bg-amber-500/10 text-amber-500 px-2 py-0.5 rounded-full font-medium flex items-center gap-1">
                                        <Shield className="w-3 h-3" /> Admin
                                    </span>
                                ) : (
                                    <span className="text-xs bg-muted text-muted-foreground px-2 py-0.5 rounded-full font-medium">
                                        User
                                    </span>
                                )}
                                <span className="text-xs text-muted-foreground">
                                    Joined {new Date(profile?.createdAt ?? "").toLocaleDateString()}
                                </span>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                {/* Edit form */}
                <Card>
                    <CardHeader>
                        <CardTitle>Edit Profile</CardTitle>
                        <CardDescription>Update your username, email, or password</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <form onSubmit={handleSave} className="flex flex-col gap-5">
                            {/* Username */}
                            <div className="flex flex-col gap-1.5">
                                <Label htmlFor="username" className="flex items-center gap-1.5">
                                    <User className="w-3.5 h-3.5" /> Username
                                </Label>
                                <Input
                                    id="username"
                                    value={username}
                                    onChange={e => setUsername(e.target.value)}
                                    required
                                />
                            </div>

                            {/* Email */}
                            <div className="flex flex-col gap-1.5">
                                <Label htmlFor="email" className="flex items-center gap-1.5">
                                    <Mail className="w-3.5 h-3.5" /> Email
                                </Label>
                                <Input
                                    id="email"
                                    type="email"
                                    value={email}
                                    onChange={e => setEmail(e.target.value)}
                                    required
                                />
                            </div>

                            <div className="border-t border-border/50 pt-4 flex flex-col gap-4">
                                <p className="text-sm font-medium flex items-center gap-1.5">
                                    <Lock className="w-3.5 h-3.5" /> Change Password
                                    <span className="text-muted-foreground font-normal">(optional)</span>
                                </p>

                                <div className="flex flex-col gap-1.5">
                                    <Label htmlFor="currentPassword">Current Password</Label>
                                    <Input
                                        id="currentPassword"
                                        type="password"
                                        placeholder="Required to change password"
                                        value={currentPassword}
                                        onChange={e => setCurrentPassword(e.target.value)}
                                    />
                                </div>
                                <div className="flex flex-col gap-1.5">
                                    <Label htmlFor="newPassword">New Password</Label>
                                    <Input
                                        id="newPassword"
                                        type="password"
                                        placeholder="Min. 8 characters"
                                        value={newPassword}
                                        onChange={e => setNewPassword(e.target.value)}
                                    />
                                </div>
                                <div className="flex flex-col gap-1.5">
                                    <Label htmlFor="confirmPassword">Confirm New Password</Label>
                                    <Input
                                        id="confirmPassword"
                                        type="password"
                                        placeholder="Repeat new password"
                                        value={confirmPassword}
                                        onChange={e => setConfirmPassword(e.target.value)}
                                    />
                                </div>
                            </div>

                            {error && (
                                <div className="flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 border border-destructive/20 rounded-md">
                                    <AlertCircle className="w-4 h-4 flex-shrink-0" />
                                    {error}
                                </div>
                            )}
                            {success && (
                                <div className="flex items-center gap-2 p-3 text-sm text-emerald-600 bg-emerald-500/10 border border-emerald-500/20 rounded-md">
                                    <CheckCircle className="w-4 h-4 flex-shrink-0" />
                                    {success}
                                </div>
                            )}

                            <Button type="submit" disabled={saving} className="w-full">
                                {saving ? "Saving…" : "Save Changes"}
                            </Button>
                        </form>
                    </CardContent>
                </Card>
            </div>
        </div>
    )
}