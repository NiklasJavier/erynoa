import { Dialog as SheetPrimitive } from 'bits-ui'
import Content from './sheet-content.svelte'
import Description from './sheet-description.svelte'
import Footer from './sheet-footer.svelte'
import Header from './sheet-header.svelte'
import Overlay from './sheet-overlay.svelte'
import Title from './sheet-title.svelte'
import Root from './sheet.svelte'

const Trigger = SheetPrimitive.Trigger
const Close = SheetPrimitive.Close
const Portal = SheetPrimitive.Portal

export {
	Root,
	Close,
	Content,
	Description,
	Footer,
	Header,
	Overlay,
	Portal,
	Title,
	Trigger,
	//
	Root as Sheet,
	Close as SheetClose,
	Content as SheetContent,
	Description as SheetDescription,
	Footer as SheetFooter,
	Header as SheetHeader,
	Overlay as SheetOverlay,
	Portal as SheetPortal,
	Title as SheetTitle,
	Trigger as SheetTrigger,
}
