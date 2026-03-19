import { auth } from "@/auth"
import Calendar from "@/components/calendar"

export default async function Page() {
    const session = await auth()
    const userId = session?.user?.id ?? null
    return <Calendar currentUserId={userId} />
}